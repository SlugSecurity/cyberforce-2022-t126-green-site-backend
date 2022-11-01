use std::{env, num::ParseIntError};

const LDAPS_SERVER_IP: &str = "LDAPS_SERVER_IP";
const LDAPS_SERVER_PORT: &str = "LDAPS_SERVER_PORT";
const FTPS_SERVER_IP: &str = "FTPS_SERVER_IP";
const FTPS_SERVER_PORT: &str = "FTPS_SERVER_PORT";
const SMTPS_SERVER_IP: &str = "SMTPS_SERVER_IP";
const SMTPS_SERVER_PORT: &str = "SMTPS_SERVER_PORT";
const DATA_HISTORIAN_IP: &str = "DATA_HISTORIAN_IP";
const DATA_HISTORIAN_PORT: &str = "DATA_HISTORIAN_PORT";
const WEB_SERVER_PORT: &str = "WEB_SERVER_PORT";
const SSL_CERTIFICATE_PEM_PATH: &str = "SSL_CERTIFICATE_PEM_PATH";
const SSL_PRIVATE_KEY_PEM_PATH: &str = "SSL_PRIVATE_KEY_PEM_PATH";

#[derive(Debug, thiserror::Error)]
pub(crate) enum BackendVarError {
    #[error("VarError encountered for the environemnt variable '{0}', {1}")]
    VarError(&'static str, #[source] env::VarError),
    #[error("Couldn't convert environment variable '{0}' to a port. Got value <{1}>. Error: {2}")]
    PortConversionError(&'static str, String, #[source] ParseIntError),
}

#[derive(Clone)]
pub(crate) struct BackendVars {
    pub ldaps_server_ip: String,
    pub ldaps_server_port: u16,
    pub ftps_server_ip: String,
    pub ftps_server_port: u16,
    pub smtps_server_ip: String,
    pub smtps_server_port: u16,
    pub data_historian_ip: String,
    pub data_historian_port: u16,
    pub web_server_port: u16,
    pub ssl_certificate_pem_path: String,
    pub ssl_private_key_pem_path: String,
}

impl BackendVars {
    fn get_str(var: &'static str) -> Result<String, BackendVarError> {
        env::var(var).map_err(|e| BackendVarError::VarError(var, e))
    }

    fn get_port(var: &'static str) -> Result<u16, BackendVarError> {
        let var_str = Self::get_str(var)?;
        var_str
            .parse()
            .map_err(|e| BackendVarError::PortConversionError(var, var_str, e))
    }

    pub fn new() -> Result<Self, BackendVarError> {
        Ok(BackendVars {
            ldaps_server_ip: Self::get_str(LDAPS_SERVER_IP)?,
            ldaps_server_port: Self::get_port(LDAPS_SERVER_PORT)?,
            ftps_server_ip: Self::get_str(FTPS_SERVER_IP)?,
            ftps_server_port: Self::get_port(FTPS_SERVER_PORT)?,
            smtps_server_ip: Self::get_str(SMTPS_SERVER_IP)?,
            smtps_server_port: Self::get_port(SMTPS_SERVER_PORT)?,
            data_historian_ip: Self::get_str(DATA_HISTORIAN_IP)?,
            data_historian_port: Self::get_port(DATA_HISTORIAN_PORT)?,
            web_server_port: Self::get_port(WEB_SERVER_PORT)?,
            ssl_certificate_pem_path: Self::get_str(SSL_CERTIFICATE_PEM_PATH)?,
            ssl_private_key_pem_path: Self::get_str(SSL_PRIVATE_KEY_PEM_PATH)?,
        })
    }
}
