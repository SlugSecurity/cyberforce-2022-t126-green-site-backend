use actix_web::{get, web::ServiceConfig, HttpResponse, Responder};

#[get("")]
async fn get_solar_data() -> impl Responder {
    HttpResponse::Ok().body("hello")
}

pub(crate) fn solar_endpoint_config(cfg: &mut ServiceConfig) {
    cfg.service(get_solar_data);
}
