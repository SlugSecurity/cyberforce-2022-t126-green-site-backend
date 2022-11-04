use actix_web::{
    post,
    web::{Json, JsonConfig, ServiceConfig},
    HttpRequest, HttpResponse,
};
use log::{error, warn};

use serde::{Deserialize, Serialize};
use sqlx::{Connection, Row, SqliteConnection};

use crate::{
    env_vars::BackendVars,
    error::{self, ErrorResponse, MISSING_APP_DATA},
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
    #[serde(skip)]
    is_valid: bool,
    token: Option<AdminToken>,
}

/// Checks that the user attempting to login has the correct credentials.
/// Returns whether the user is an admin or not if credentials could be validated.
async fn check_credentials(
    user_login: &UserLogin,
    vars: &BackendVars,
) -> sqlx::Result<Authentication> {
    // CRITICAL CODE: ENSURE AN OK RESULT CAN NEVER BE RETURNED IF THE CREDENTIALS ARE INVALID.

    let mut conn = SqliteConnection::connect(&vars.sqlite_file_name).await?;
    let pwd: Option<String> = sqlx::query("SELECT password FROM users WHERE username=?;")
        .bind(user_login.username.as_str())
        .fetch_optional(&mut conn)
        .await?
        .and_then(|pwd| pwd.try_get(0).ok())
        .filter(|pwd| pwd == &user_login.password);
    let (is_valid, used_admin_username) = (
        pwd.is_some(),
        user_login.username == vars.admin_account_username,
    );

    Ok(Authentication {
        is_valid,
        token: (is_valid && used_admin_username).then(|| AdminToken::new(vars.admin_token.clone())),
    })
}

#[post("")]
async fn login(req: HttpRequest, user_login: Json<UserLogin>) -> HttpResponse {
    if !(MIN_USERNAME_LEN..MAX_USERNAME_LEN).contains(&user_login.username.len()) {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: format!(
                "Username must be between {MIN_USERNAME_LEN} and {MAX_USERNAME_LEN} characters."
            ),
        });
    } else if !(MIN_PASSWORD_LEN..MAX_PASSWORD_LEN).contains(&user_login.password.len()) {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: format!(
                "Password must be between {MIN_PASSWORD_LEN} and {MAX_PASSWORD_LEN} characters."
            ),
        });
    } else if !user_login.username.bytes().all(|b| b.is_ascii_lowercase()) {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: format!("Username must be all lowercase ASCII charcters."),
        });
    }

    if let Some(vars) = req.app_data::<BackendVars>() {
        match check_credentials(&user_login.0, &vars).await {
            Ok(Authentication {
                is_valid: false, ..
            }) => {
                warn!(
                    "{:?} tried logging in with bad credentials. Username: {}. Password: {}",
                    req.connection_info().peer_addr(),
                    user_login.0.username,
                    user_login.0.password,
                );

                HttpResponse::BadRequest().json(ErrorResponse {
                    error: "Bad credentials.".to_string(),
                })
            }
            Ok(authed) => HttpResponse::Ok().json(authed),
            Err(err) => {
                error!("Encountered sqlx error: {err}. Sending 500 response code...");

                error::internal_server_error()
            }
        }
    } else {
        error!(
            "{MISSING_APP_DATA}. BackendVars: {:?}",
            req.app_data::<BackendVars>(),
        );

        error::internal_server_error()
    }
}

pub(crate) fn login_endpoint_config(cfg: &mut ServiceConfig) {
    let json_cfg = JsonConfig::default()
        .limit(MAX_USERNAME_LEN + MAX_PASSWORD_LEN + BUFFER_SPACE)
        .content_type(|mime_type| mime_type == mime::APPLICATION_JSON);

    cfg.service(login).app_data(json_cfg);
}
