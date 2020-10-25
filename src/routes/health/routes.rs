use crate::{
    health::HealthChecker,
    routes::health::models::{HealthResponse, HealthResponseStatus},
};
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::mysql;

#[get("")]
pub async fn get(
    database_pool: web::Data<mysql::MySqlPool>,
    http_client_pool: web::Data<reqwest::Client>,
) -> impl Responder {
    let health = HealthChecker::new(database_pool.get_ref(), http_client_pool.get_ref())
        .check()
        .await;

    match HealthResponse::from(&health) {
        HealthResponseStatus::Ok(response) => HttpResponse::Ok().json(response),
        HealthResponseStatus::InternalServerError(response) => {
            HttpResponse::InternalServerError().json(response)
        }
    }
}
