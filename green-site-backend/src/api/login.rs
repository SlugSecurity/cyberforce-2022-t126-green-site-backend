use actix_web::{
    post,
    web::{Json, JsonConfig, ServiceConfig},
    HttpResponse,
};
use serde::{Deserialize, Serialize};

use crate::error::ErrorResponse;

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
    token: u128,
}

/// Checks that the user attempting to login has the correct credentials.
/// Returns an error response if credentials couldn't be validated.
/// Returns whether the user is an admin or not if credentials could be validated.
async fn check_credentials(user_login: UserLogin) -> Result<bool, HttpResponse> {
    // CRITICAL CODE: ENSURE AN OK RESULT CAN NEVER BE RETURNED IF THE CREDENTIALS ARE INVALID.

    todo!()
}

#[post("")]
async fn login(user_login: Json<UserLogin>) -> HttpResponse {
    if !(MIN_USERNAME_LEN..MAX_USERNAME_LEN).contains(&user_login.username.len()) {
        let name_len_err = ErrorResponse(format!(
            "Username must be between {MIN_USERNAME_LEN} and {MAX_USERNAME_LEN} characters."
        ));
        return HttpResponse::BadRequest().json(name_len_err);
    } else if !(MIN_PASSWORD_LEN..MAX_PASSWORD_LEN).contains(&user_login.password.len()) {
        let bad_pass_err = ErrorResponse(format!(
            "Password must be between {MIN_PASSWORD_LEN} and {MAX_PASSWORD_LEN} characters."
        ));
        return HttpResponse::BadRequest().json(bad_pass_err);
    }

    if let Err(err_response) = check_credentials(user_login.0).await {
        return err_response;
    }

    // 2.) if credentials are correct, generate token and SHA-256
    // 3.) send LoginResponse struct
    HttpResponse::Ok().body("hello")
}

pub(crate) fn login_endpoint_config(cfg: &mut ServiceConfig) {
    let json_cfg = JsonConfig::default()
        .limit(MAX_USERNAME_LEN + MAX_PASSWORD_LEN + BUFFER_SPACE)
        .content_type(|mime_type| mime_type == mime::APPLICATION_JSON);

    cfg.service(login).app_data(json_cfg);
}
