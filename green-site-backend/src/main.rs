use std::{
    error::Error,
    fs::File,
    io::{BufReader, Read},
    sync::Arc,
    time::Duration,
};

use actix_web::{
    middleware::{self, Logger, TrailingSlash},
    web, App, HttpServer,
};
use env_logger::Builder;
use env_vars::BackendVars;
use error::CertConfigError;
use log::LevelFilter;
use rustls::{Certificate, ClientConfig, PrivateKey, RootCertStore, ServerConfig};
use rustls_pemfile::Item::*;
use sqlx::{
    mysql::{MySqlConnectOptions, MySqlSslMode},
    pool::PoolOptions,
    MySqlPool,
};

use suppaftp::async_native_tls::Certificate as FtpCertificate;

mod api;
mod env_vars;
mod error;
mod token;

fn get_web_server_cert(vars: &BackendVars) -> Result<ServerConfig, CertConfigError> {
    use CertConfigError::*;

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth();

    let cert_path = vars.ssl_certificate_pem_path.as_str();
    let cert_file = match File::open(cert_path) {
        Ok(ok) => ok,
        Err(err) => return Err(ReadPemIoError(cert_path.to_string(), err)),
    };
    let key_path = vars.ssl_private_key_pem_path.as_str();
    let key_file = match File::open(key_path) {
        Ok(ok) => ok,
        Err(err) => return Err(ReadPemIoError(key_path.to_string(), err)),
    };
    let cert_chain = rustls_pemfile::certs(&mut BufReader::new(cert_file))
        .map_err(|e| ReadPemIoError(cert_path.to_string(), e))?
        .into_iter()
        .map(Certificate)
        .collect::<Vec<_>>();

    let key_item = rustls_pemfile::read_one(&mut BufReader::new(key_file))
        .map_err(|e| ReadPemIoError(key_path.to_string(), e))?
        .ok_or_else(|| UnrecognizedPrivateKey(key_path.to_string()))?;
    let key = match key_item {
        X509Certificate(k) | RSAKey(k) | PKCS8Key(k) | ECKey(k) => PrivateKey(k),
        _ => return Err(UnrecognizedPrivateKey(key_path.to_string())),
    };

    Ok(config.with_single_cert(cert_chain, key)?)
}

fn get_trusted_roots(
    vars: &BackendVars,
) -> Result<(RootCertStore, FtpCertificate), CertConfigError> {
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

    let native_root_cert = FtpCertificate::from_pem(&root_cert_bytes)?;
    let rust_root_cert = rustls_pemfile::certs(&mut &root_cert_bytes[..])
        .map_err(|e| ReadPemIoError(root_cert_path.to_string(), e))?
        .into_iter()
        .map(Certificate)
        .next()
        .ok_or_else(|| BadRootCertificate(None, None))?;

    let mut cert_store = RootCertStore::empty();
    cert_store.add(&rust_root_cert)?;

    Ok((cert_store, native_root_cert))
}

fn create_pool(vars: &BackendVars) -> MySqlPool {
    let conn_options = MySqlConnectOptions::new()
        .host(&vars.data_historian_ip)
        .port(vars.data_historian_port)
        .username(&vars.data_historian_user)
        .password(&vars.data_historian_pass)
        .database(&vars.data_historian_db_name)
        .ssl_mode(MySqlSslMode::VerifyCa)
        .ssl_ca(&vars.root_certificate_path);

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
    let rustls_server_config = get_web_server_cert(&backend_vars)?;
    let port = backend_vars.web_server_port;
    let (root_store, native_cert) = get_trusted_roots(&backend_vars)?;

    let rustls_client_config = Arc::new(
        ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth(),
    );

    let mysql_pool = create_pool(&backend_vars);

    Builder::new()
        .filter_level(LevelFilter::Warn)
        .filter_module("actix_web::middleware::logger", LevelFilter::Info)
        .init();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(middleware::NormalizePath::new(TrailingSlash::Trim))
            .app_data(backend_vars.clone())
            .app_data(mysql_pool.clone())
            .app_data(rustls_client_config.clone())
            .app_data(native_cert.clone())
            .service(web::scope("/api").configure(api::endpoint_config))
    })
    .bind_rustls(format!("0.0.0.0:{port}"), rustls_server_config)?
    .run()
    .await?;

    Ok(())
}
