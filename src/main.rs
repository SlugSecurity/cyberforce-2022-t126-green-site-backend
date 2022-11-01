use std::{
    error::Error,
    fs::File,
    io::{self, BufReader},
};

use actix_web::{
    middleware::{self, TrailingSlash},
    web, App, HttpServer,
};
use env_vars::BackendVars;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::Item::*;

mod api;
mod env_vars;

// check out sending large post request on invalid endpoint: see https://github.com/actix/actix-web/issues/2906
// check if streamed responses cause other requests to fail: https://github.com/actix/actix-web/issues/2774

// make sure to enforce lower or higher payload limits if necessary (there's one by default)

// TODO: add logging to monitor requests

#[derive(Debug, thiserror::Error)]
enum ServerConfigError {
    #[error("Error while reading PEM file: '{0}'")]
    ReadPemIoError(String, #[source] io::Error),

    #[error("Error while setting certificate and private key. Check that they are correct.")]
    SetCertificateError(#[from] rustls::Error),

    #[error("Unrecognized or no private key in {0}")]
    UnrecognizedPrivateKey(String),
}

fn get_cert(vars: &BackendVars) -> Result<ServerConfig, ServerConfigError> {
    use ServerConfigError::*;

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
        .collect();
    let key_item = rustls_pemfile::read_one(&mut BufReader::new(key_file))
        .map_err(|e| ReadPemIoError(key_path.to_string(), e))?
        .ok_or_else(|| UnrecognizedPrivateKey(key_path.to_string()))?;
    let key = match key_item {
        X509Certificate(k) | RSAKey(k) | PKCS8Key(k) | ECKey(k) => PrivateKey(k),
        _ => return Err(UnrecognizedPrivateKey(key_path.to_string())),
    };

    Ok(config.with_single_cert(cert_chain, key)?)
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let backend_vars = BackendVars::new()?;
    let rustls_config = get_cert(&backend_vars)?;
    let port = backend_vars.web_server_port;

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::NormalizePath::new(TrailingSlash::Trim))
            .app_data(backend_vars.clone())
            .service(web::scope("/api").configure(api::endpoint_config))
    })
    .bind_rustls(format!("0.0.0.0:{port}"), rustls_config)?
    .run()
    .await?;

    Ok(())
}
