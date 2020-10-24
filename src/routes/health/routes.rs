use actix_web::{get, HttpResponse, Responder};
use std::collections::HashMap;

#[get("")]
pub async fn get() -> impl Responder {
    HttpResponse::Ok()
}
