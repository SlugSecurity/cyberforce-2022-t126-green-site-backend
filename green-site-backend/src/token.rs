use actix_web::{http::header, HttpRequest};
use serde::Serialize;

use crate::env_vars::BackendVars;

#[derive(Serialize)]
#[serde(transparent)]
pub(crate) struct AdminToken(String);

impl AdminToken {
    pub fn new(token: String) -> Self {
        Self(token)
    }
}

/// Verifies the token is that of the admin account.
pub(crate) fn has_admin_token(req: HttpRequest, env_var: BackendVars) -> bool {
    let auth_header = req.headers().get(header::AUTHORIZATION);

    auth_header
        .and_then(|auth| auth.to_str().ok())
        .filter(|auth_str| {
            auth_str.starts_with("Bearer ")
                && auth_str.get(7..) == Some(env_var.admin_token.as_str())
        })
        .is_some()
}
