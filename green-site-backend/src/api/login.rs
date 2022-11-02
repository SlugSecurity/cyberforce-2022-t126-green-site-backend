use actix_web::{get, web::ServiceConfig, HttpResponse, Responder};

#[get("")]
async fn login() -> impl Responder {
    HttpResponse::Ok().body("hello")
}

pub(crate) fn login_endpoint_config(cfg: &mut ServiceConfig) {
    cfg.service(login);
}
