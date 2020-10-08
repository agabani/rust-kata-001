use actix_web::web::ServiceConfig;
use actix_web::{get, post, web, HttpResponse, Responder};

#[get("/")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
pub async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

pub async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hello there!")
}

pub fn configure(service_config: &mut ServiceConfig) {
    service_config
        .service(hello)
        .service(echo)
        .route("/hey", web::get().to(manual_hello));
}
