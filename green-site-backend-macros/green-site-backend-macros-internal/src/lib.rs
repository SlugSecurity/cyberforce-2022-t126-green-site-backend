use std::{env, error::Error, fmt::Display, num::ParseIntError};
use EnvVarParseError::*;

/// Error type for parsing an environment variable.
#[derive(Debug, Clone)]
pub enum EnvVarParseError {
    EnvVarError(&'static str, env::VarError),
    NumConversionError(&'static str, String, ParseIntError),
}

impl Display for EnvVarParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnvVarError(name, err) => {
                write!(
                    f,
                    "VarError encountered for the env variable '{name}', {err}"
                )
            }
            NumConversionError(name, val, err) => write!(
                f,
                "Couldn't convert env variable '{name}' to a port. Got value <{val}>. Error: {err}"
            ),
        }
    }
}

impl Error for EnvVarParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            EnvVarError(_, err) => Some(err),
            NumConversionError(_, _, err) => Some(err),
        }
    }
}
