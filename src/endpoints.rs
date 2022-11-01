use actix_web::{get, post, web::ServiceConfig, HttpResponse, Responder};

#[get("")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("hello")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

pub(crate) fn endpoint_config(cfg: &mut ServiceConfig) {
    cfg.service(hello).service(echo);
}
