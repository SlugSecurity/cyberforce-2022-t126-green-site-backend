use actix_web::{get, post, web::ServiceConfig, HttpResponse, Responder};

#[get("")]
async fn get_emails() -> impl Responder {
    HttpResponse::Ok().body("")
}

#[post("")]
async fn upload_emails() -> impl Responder {
    HttpResponse::Ok().body("")
}

pub(crate) fn email_endpoint_config(cfg: &mut ServiceConfig) {
    cfg.service(get_emails).service(upload_emails);
}
