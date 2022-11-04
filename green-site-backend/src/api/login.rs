use actix_web::{
    post,
    web::{Json, JsonConfig, ServiceConfig},
    HttpRequest, HttpResponse,
};
use ldap3::{drive, LdapConnAsync, LdapError, LdapResult};
use log::{error, warn};
use serde::{Deserialize, Serialize};

use crate::{
    env_vars::BackendVars,
    error::{ErrorResponse, INTERNAL_ERROR, NO_ENV_VAR_APP_DATA},
    token::AdminToken,
};

const MIN_USERNAME_LEN: usize = 1;
const MIN_PASSWORD_LEN: usize = 1;
const MAX_USERNAME_LEN: usize = 72;
const MAX_PASSWORD_LEN: usize = 72;
const BUFFER_SPACE: usize = 50;

#[derive(Deserialize)]
struct UserLogin {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct Authentication {
    is_admin: bool,
    token: Option<AdminToken>,
}

const BAD_CRED_CODE: u32 = 49; // Invalid credential result code for LDAP. See https://www.rfc-editor.org/rfc/rfc4511#appendix-A.1
const LDAP_DIR_USERS_DIR: &str = "CN=Users,CN=GreenTeamWebsite,DC=sunpartners,DC=local";

/// Checks that the user attempting to login has the correct credentials.
/// Returns whether the user is an admin or not if credentials could be validated.
async fn check_credentials(user_login: &UserLogin, vars: &BackendVars) -> Result<bool, LdapError> {
    // CRITICAL CODE: ENSURE AN OK RESULT CAN NEVER BE RETURNED IF THE CREDENTIALS ARE INVALID.

    let conn_uri = format!("ldaps://{}", vars.ldaps_server_ip);
    let (conn, mut ldap) = LdapConnAsync::new(conn_uri.as_str()).await?;

    drive!(conn);

    let user_name = format!("CN={},{LDAP_DIR_USERS_DIR}", user_login.username);
    ldap.simple_bind(user_name.as_str(), user_login.password.as_str())
        .await?
        .success()?; // Propogates the ldapresult if it wasnt successful

    Ok(vars.admin_account_username == user_name) // Return if user is admin
}

#[post("")]
async fn login(req: HttpRequest, user_login: Json<UserLogin>) -> HttpResponse {
    if !(MIN_USERNAME_LEN..MAX_USERNAME_LEN).contains(&user_login.username.len()) {
        return HttpResponse::BadRequest().json(ErrorResponse(format!(
            "Username must be between {MIN_USERNAME_LEN} and {MAX_USERNAME_LEN} characters."
        )));
    } else if !(MIN_PASSWORD_LEN..MAX_PASSWORD_LEN).contains(&user_login.password.len()) {
        return HttpResponse::BadRequest().json(ErrorResponse(format!(
            "Password must be between {MIN_PASSWORD_LEN} and {MAX_PASSWORD_LEN} characters."
        )));
    } else if !user_login.username.bytes().all(|b| b.is_ascii_lowercase()) {
        return HttpResponse::BadRequest().json(ErrorResponse(format!(
            "Username must be all lowercase ASCII charcters."
        )));
    }

    if let Some(vars) = req.app_data::<BackendVars>() {
        match check_credentials(&user_login.0, &vars).await {
            Err(LdapError::LdapResult {
                result: LdapResult {
                    rc: BAD_CRED_CODE, ..
                },
            }) => {
                warn!(
                    "{:?} tried logging in with bad credentials. Username: {}. Password: {}",
                    req.connection_info().peer_addr(),
                    user_login.0.username,
                    user_login.0.password,
                );

                HttpResponse::BadRequest().json(ErrorResponse("Bad credentials.".to_string()))
            }
            Err(err) => {
                error!("Encountered LDAP error: {err}. Sending 500 response code...");

                HttpResponse::BadRequest().json(ErrorResponse(INTERNAL_ERROR.to_string()))
            }
            Ok(is_admin) => HttpResponse::Ok().json(Authentication {
                is_admin,
                token: is_admin.then(|| AdminToken::new(vars.admin_token.clone())),
            }),
        }
    } else {
        error!("{NO_ENV_VAR_APP_DATA}");

        HttpResponse::BadRequest().json(ErrorResponse(INTERNAL_ERROR.to_string()))
    }
}

pub(crate) fn login_endpoint_config(cfg: &mut ServiceConfig) {
    let json_cfg = JsonConfig::default()
        .limit(MAX_USERNAME_LEN + MAX_PASSWORD_LEN + BUFFER_SPACE)
        .content_type(|mime_type| mime_type == mime::APPLICATION_JSON);

    cfg.service(login).app_data(json_cfg);
}
