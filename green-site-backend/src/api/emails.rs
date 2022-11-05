use std::{error::Error, time::Duration};

use actix_web::{
    get, post,
    web::{Json, ServiceConfig},
    HttpRequest, HttpResponse, Responder,
};
use lettre::{
    message::Mailbox,
    transport::smtp::{
        authentication::Credentials,
        client::{Certificate, Tls, TlsParameters},
        Error as SmtpError,
    },
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use log::error;
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

fn get_var_and_mail_cert(req: &HttpRequest) -> Option<(&BackendVars, &Certificate)> {
    Some((req.app_data()?, req.app_data()?))
}

macro_rules! verify_var_cert {
    ($req:ident) => {
        match get_var_and_mail_cert(&$req) {
            Some(pair) => pair,
            None => return crate::error::internal_server_error(),
        }
    };
}

#[get("")]
async fn get_emails(req: HttpRequest) -> impl Responder {
    let (vars, cert) = verify_var_cert!(req);

    verify_admin_token!(req, vars);

    HttpResponse::Ok().body("")
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

    let (vars, cert) = verify_var_cert!(req);

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
