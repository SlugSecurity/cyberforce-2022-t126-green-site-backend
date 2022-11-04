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
pub(crate) fn has_admin_token(token: AdminToken, env_var: BackendVars) -> bool {
    env_var.admin_token == token.0
}
