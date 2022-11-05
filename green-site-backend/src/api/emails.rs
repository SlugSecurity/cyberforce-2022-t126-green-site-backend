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

#[derive(Debug, Serialize, Deserialize)]
struct Email {
    subject: String,
    from_name: String,
    from_email: String,
    body: String,
}

impl Email {
    fn to_message(self, vars: &BackendVars) -> Result<Message, lettre::error::Error> {
        Message::builder()
            .from(
                Mailbox::try_from((self.from_name, self.from_email))
                    .map_err(|_| lettre::error::Error::MissingFrom)?,
            )
            .to(
                Mailbox::try_from(("Web", vars.email_user.clone())).map_err(|_| {
                    println!("fsfje");
                    lettre::error::Error::MissingTo
                })?,
            )
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

fn parse_headers(headers: String) -> Email {
    let mut subject = None;
    let mut name_email = None;

    for header in headers.lines() {
        if header.starts_with("SUBJECT: ") && header.len() > 9 {
            subject = Some(header[9..].to_string());
        } else if header.starts_with("FROM: ") && header.len() > 6 {
            name_email = header.rfind(' ').map(|s| header[6..].split_at(s - 6));
        }
    }

    let (name, email) = match name_email {
        Some((name, email)) => (name, email),
        _ => {
            return Email {
                subject: subject.unwrap_or_else(String::new),
                from_name: name_email.unwrap_or(("", "")).0.to_string(),
                from_email: name_email.unwrap_or(("", "")).1.trim_start().to_string(),
                body: String::new(),
            }
        }
    };

    Email {
        subject: subject.unwrap_or_else(String::new),
        from_name: name.to_string(),
        from_email: email.trim_start().to_string(),
        body: String::new(),
    }
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

        let mut session = conn
            .login(vars.email_user.as_str(), vars.email_pass.as_str())
            .map_err(|(err, _)| err)?;
        let inbox = session.select("INBOX")?;
        let mail_count = inbox.exists;

        let mut emails = Vec::new();

        for re in 1..=mail_count {
            let data = session.fetch(
                re.to_string(),
                "(body[HEADER.FIELDS (SUBJECT FROM)] body[text])",
            )?;

            for query in data.iter() {
                if let (Some(header), Some(text)) = (query.header(), query.text()) {
                    let mut mail = parse_headers(String::from_utf8_lossy(header).into_owned());
                    mail.body = String::from_utf8_lossy(text).trim_end().to_string();

                    emails.push(mail);
                }
            }
        }

        Ok(emails)
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
            .dangerous_accept_invalid_hostnames(true)
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
