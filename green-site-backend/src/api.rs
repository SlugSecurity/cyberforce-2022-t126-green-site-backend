use actix_web::web::{self, ServiceConfig};

use self::{
    emails::email_endpoint_config, files::file_endpoint_config, login::login_endpoint_config,
    solar::solar_endpoint_config,
};

mod emails;
mod files;
mod login;
mod solar;

pub(crate) fn endpoint_config(cfg: &mut ServiceConfig) {
    cfg.service(web::scope("/files").configure(file_endpoint_config))
        .service(web::scope("/emails").configure(email_endpoint_config))
        .service(web::scope("/login").configure(login_endpoint_config))
        .service(web::scope("/solar").configure(solar_endpoint_config));
}
