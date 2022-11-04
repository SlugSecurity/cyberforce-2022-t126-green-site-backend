use actix_web::HttpResponse;
use serde::Serialize;
use std::{error::Error, fmt::Display, io};
use suppaftp::async_native_tls;
use CertConfigError::*;

const INTERNAL_ERROR: &str = "Internal server error encountered. Please try again later.";

pub(crate) const MISSING_APP_DATA: &str = "Missing app data. This should never happen.";

#[derive(Debug)]
pub(crate) enum CertConfigError {
    ReadPemIoError(String, io::Error),
    SetCertificateError(rustls::Error),
    UnrecognizedPrivateKey(String),
    BadRootCertificate(Option<webpki::Error>, Option<async_native_tls::Error>),
}

impl Display for CertConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadPemIoError(file, _) => write!(f, "Error while reading PEM file: {file}"),
            SetCertificateError(_) => write!(f, "Certificate or private key not valid."),
            UnrecognizedPrivateKey(s) => write!(f, "Unrecognized or no private key in {s}"),
            BadRootCertificate(err, _) => write!(f, "Bad root certificate provided: {err:?}"),
        }
    }
}

impl From<rustls::Error> for CertConfigError {
    fn from(value: rustls::Error) -> Self {
        SetCertificateError(value)
    }
}

impl From<webpki::Error> for CertConfigError {
    fn from(value: webpki::Error) -> Self {
        BadRootCertificate(Some(value), None)
    }
}

impl From<async_native_tls::Error> for CertConfigError {
    fn from(value: async_native_tls::Error) -> Self {
        BadRootCertificate(None, Some(value))
    }
}

impl Error for CertConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ReadPemIoError(_, err) => Some(err),
            SetCertificateError(err) => Some(err),
            _ => None,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct ErrorResponse {
    pub error: String,
}

pub(crate) fn internal_server_error() -> HttpResponse {
    HttpResponse::InternalServerError().json(ErrorResponse {
        error: INTERNAL_ERROR.to_string(),
    })
}
