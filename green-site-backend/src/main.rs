use std::{error::Error, fs::File, io::Read, time::Duration};

use actix_web::{
    middleware::{self, Logger, TrailingSlash},
    web, App, HttpServer,
};
use env_logger::Builder;
use env_vars::BackendVars;
use error::CertConfigError;
use lettre::transport::smtp::client::Certificate as SmtpCertificate;
use log::LevelFilter;
use native_tls::{Protocol, TlsConnector};
use sqlx::{mysql::MySqlConnectOptions, pool::PoolOptions, MySqlPool};
use suppaftp::async_native_tls::Certificate as FtpCertificate;

mod api;
mod env_vars;
mod error;
mod token;

fn get_trusted_roots(
    vars: &BackendVars,
) -> Result<(FtpCertificate, SmtpCertificate), CertConfigError> {
    use CertConfigError::*;

    let root_cert_path = vars.root_certificate_path.as_str();
    let mut root_cert_file = match File::open(root_cert_path) {
        Ok(ok) => ok,
        Err(err) => return Err(ReadPemIoError(root_cert_path.to_string(), err)),
    };
    let mut root_cert_bytes = Vec::new();
    root_cert_file
        .read_to_end(&mut root_cert_bytes)
        .map_err(|e| ReadPemIoError(root_cert_path.to_string(), e))?;

    Ok((
        FtpCertificate::from_pem(&root_cert_bytes)?,
        SmtpCertificate::from_pem(&root_cert_bytes)?,
    ))
}

fn create_pool(vars: &BackendVars) -> MySqlPool {
    let conn_options = MySqlConnectOptions::new()
        .host(&vars.data_historian_ip)
        .port(vars.data_historian_port)
        .username(&vars.data_historian_user)
        .password(&vars.data_historian_pass)
        .database(&vars.data_historian_db_name);

    PoolOptions::new()
        .max_connections(50)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(3))
        .max_lifetime(Some(Duration::from_secs(3600)))
        .connect_lazy_with(conn_options)
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let backend_vars = BackendVars::new()?;
    let port = backend_vars.web_server_port;
    let (native_cert, smtp_cert) = get_trusted_roots(&backend_vars)?;
    let mysql_pool = create_pool(&backend_vars);
    let connector = TlsConnector::builder()
        .min_protocol_version(Some(Protocol::Tlsv12))
        .add_root_certificate(native_cert.clone())
        .use_sni(false)
        .build()?;

    Builder::new()
        .filter_level(LevelFilter::Warn)
        .filter_module("actix_web::middleware::logger", LevelFilter::Info)
        .init();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new(
                r#"%{r}a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#,
            ))
            .wrap(middleware::NormalizePath::new(TrailingSlash::Trim))
            .app_data(backend_vars.clone())
            .app_data(mysql_pool.clone())
            .app_data(smtp_cert.clone())
            .app_data(native_cert.clone())
            .app_data(connector.clone())
            .service(web::scope("/api").configure(api::endpoint_config))
    })
    .bind(format!("0.0.0.0:{port}"))?
    .run()
    .await?;

    Ok(())
}
