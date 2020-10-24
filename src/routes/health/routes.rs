use crate::{
    health::{
        database_health_checker::DatabaseHealthChecker,
        internet_http_health_checker::InternetHttpHealthChecker,
        internet_https_health_checker::InternetHttpsHealthChecker,
        uptime_health_checker::UptimeHealthChecker, Health, HealthChecker,
    },
    routes::health::models::HealthResponse,
};
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::mysql;

#[get("")]
pub async fn get(
    database_pool: web::Data<mysql::MySqlPool>,
    http_client_pool: web::Data<reqwest::Client>,
) -> impl Responder {
    let uptime = UptimeHealthChecker::new();
    let database = DatabaseHealthChecker::new(database_pool.get_ref());
    let internet_http_health_checker = InternetHttpHealthChecker::new(http_client_pool.get_ref());
    let internet_https_health_checker = InternetHttpsHealthChecker::new(http_client_pool.get_ref());

    let health_checks = futures::future::join_all(vec![
        uptime.check(),
        database.check(),
        internet_http_health_checker.check(),
        internet_https_health_checker.check(),
    ])
    .await;

    let health = Health::from(&health_checks);

    let response = HealthResponse::from(&health);

    HttpResponse::Ok().json(response)
}
