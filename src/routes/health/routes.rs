use crate::{
    health::{
        DatabaseHealthChecker, Health, HealthChecker, InternetHttpHealthChecker,
        InternetHttpsHealthChecker, UptimeHealthChecker,
    },
    routes::health::models::{HealthResponse, HealthResponseStatus},
};
use actix_web::{get, web, HttpResponse, Responder};
use futures::future::join_all;
use sqlx::mysql;

#[get("")]
pub async fn get(
    database_pool: web::Data<mysql::MySqlPool>,
    http_client_pool: web::Data<reqwest::Client>,
) -> impl Responder {
    let health_checks = join_all(vec![
        UptimeHealthChecker::new().check(),
        DatabaseHealthChecker::new(database_pool.get_ref()).check(),
        InternetHttpHealthChecker::new(http_client_pool.get_ref()).check(),
        InternetHttpsHealthChecker::new(http_client_pool.get_ref()).check(),
    ])
    .await;

    let health = Health::from(&health_checks);

    let response = HealthResponse::from(&health);

    match response {
        HealthResponseStatus::Pass(response) => HttpResponse::Ok().json(response),
        HealthResponseStatus::Fail(response) => HttpResponse::InternalServerError().json(response),
        HealthResponseStatus::Warn(response) => HttpResponse::Ok().json(response),
    }
}
