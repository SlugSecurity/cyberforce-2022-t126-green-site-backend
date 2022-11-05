use std::{error::Error, time::Duration};

use actix_web::{
    get, post,
    rt::task,
    web::{Json, ServiceConfig},
    HttpRequest, HttpResponse, Responder,
};
use lettre::{
    message::Mailbox,
    transport::smtp::{
        authentication::Credentials,
        client::{Certificate, Tls, TlsParameters},
    },
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use log::error;
use native_tls::TlsConnector;
use serde::{Deserialize, Serialize};

use crate::{env_vars::BackendVars, error::internal_server_error, verify_admin_token};

#[derive(Serialize, Deserialize)]
struct Email {
    subject: String,
    from: Mailbox,
    body: String,
    // use to but not derserialized, we intiialize it
}

impl Email {
    fn to_message(self, vars: &BackendVars) -> Result<Message, lettre::error::Error> {
        Message::builder()
            .from(self.from)
            .to(Mailbox::try_from(("Web", vars.email_user.clone()))
                .map_err(|_| lettre::error::Error::MissingTo)?)
            .subject(self.subject)
            .body(self.body)
    }
}

fn get_two_vars<'a, T: 'static, U: 'static>(req: &'a HttpRequest) -> Option<(&'a T, &'a U)> {
    Some((req.app_data()?, req.app_data()?))
}

macro_rules! verify_two_vars {
    ($req:ident) => {
        match get_two_vars(&$req) {
            Some(pair) => pair,
            None => return internal_server_error(),
        }
    };
}

#[get("")]
async fn get_emails(req: HttpRequest) -> impl Responder {
    fn imap_emails(
        connector: &TlsConnector,
        vars: &BackendVars,
    ) -> Result<Vec<Email>, imap::Error> {
        let conn = imap::connect_starttls(
            (vars.email_server_ip.as_str(), vars.imap_server_port),
            vars.email_server_ip.as_str(),
            connector,
        )?;

        conn.login(vars.email_user.as_str(), vars.email_pass.as_str())
            .map_err(|(err, _)| err)?;

        todo!()
    }

    let (vars, connector): (&BackendVars, &TlsConnector) = verify_two_vars!(req);
    let (vars, connector) = (vars.clone(), connector.clone());

    verify_admin_token!(req, vars);

    let emails_task = task::spawn_blocking(move || imap_emails(&connector, &vars));

    match emails_task.await {
        Ok(Ok(emails)) => HttpResponse::Ok().json(emails),
        Ok(Err(err)) => {
            error!("Encountered error while fetching emails via IMAP: {err}");

            internal_server_error()
        }
        Err(err) => {
            error!("Encountered JoinError: {err}");

            internal_server_error()
        }
    }
}

#[post("")]
async fn upload_email(req: HttpRequest, email: Json<Email>) -> impl Responder {
    async fn smtp_upload(
        email: Email,
        vars: &BackendVars,
        cert: &Certificate,
    ) -> Result<(), Box<dyn Error>> {
        let tls_params = TlsParameters::builder(vars.email_server_ip.clone())
            .add_root_certificate(cert.clone())
            .build_native()?;
        let transport =
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(vars.email_server_ip.clone())
                .port(vars.smtp_server_port)
                .tls(Tls::Required(tls_params))
                .timeout(Some(Duration::from_secs(3)))
                .credentials(Credentials::new(
                    vars.email_user.clone(),
                    vars.email_pass.clone(),
                ))
                .build();

        transport.send(email.to_message(vars)?).await?;

        Ok(())
    }

    let (vars, cert) = verify_two_vars!(req);

    match smtp_upload(email.0, vars, cert).await {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(err) => {
            error!("Error encountered uploading email to mail server: {err}");

            internal_server_error()
        }
    }
}

pub(crate) fn email_endpoint_config(cfg: &mut ServiceConfig) {
    cfg.service(get_emails).service(upload_email);
}
