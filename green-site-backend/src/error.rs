use serde::Serialize;
use std::{error::Error, fmt::Display, io};
use ServerConfigError::*;

pub(crate) const INTERNAL_ERROR: &str =
    "Internal server error encountered. Please try again later.";

pub(crate) const NO_ENV_VAR_APP_DATA: &str =
    "Couldn't find any environment variable app data. This should never happen.";

#[derive(Debug)]
pub(crate) enum ServerConfigError {
    ReadPemIoError(String, io::Error),
    SetCertificateError(rustls::Error),
    UnrecognizedPrivateKey(String),
}

impl Display for ServerConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadPemIoError(file, _) => write!(f, "Error while reading PEM file: {file}"),
            SetCertificateError(_) => write!(f, "Certificate or private key not valid."),
            UnrecognizedPrivateKey(s) => write!(f, "Unrecognized or no private key in {s}"),
        }
    }
}

impl From<rustls::Error> for ServerConfigError {
    fn from(value: rustls::Error) -> Self {
        SetCertificateError(value)
    }
}

impl Error for ServerConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ReadPemIoError(_, err) => Some(err),
            SetCertificateError(err) => Some(err),
            UnrecognizedPrivateKey(_) => None,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct ErrorResponse(#[serde(rename(serialize = "error"))] pub String);
