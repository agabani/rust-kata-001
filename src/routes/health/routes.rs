use crate::health::HealthChecker;
use crate::routes::health::models::HealthResponse;
use crate::routes::health::models::HealthResponseStatus;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::mysql;

const CONTENT_TYPE_HEADER: &str = "content-type";
const CONTENT_TYPE_VALUE: &str = "application/health+json";

#[get("")]
pub(crate) async fn get(
    database_pool: web::Data<mysql::MySqlPool>,
    http_client_pool: web::Data<reqwest::Client>,
    redis_pool: web::Data<redis::aio::MultiplexedConnection>,
) -> impl Responder {
    let health = HealthChecker::new(
        database_pool.get_ref(),
        http_client_pool.get_ref(),
        redis_pool.get_ref(),
    )
    .check()
    .await;

    match HealthResponse::from(&health) {
        HealthResponseStatus::Ok(response) => HttpResponse::Ok()
            .json(response)
            .with_header(CONTENT_TYPE_HEADER, CONTENT_TYPE_VALUE),
        HealthResponseStatus::InternalServerError(response) => HttpResponse::InternalServerError()
            .json(response)
            .with_header(CONTENT_TYPE_HEADER, CONTENT_TYPE_VALUE),
    }
}
