use crate::health::{
    DatabaseHealthChecker, Health, HealthChecker, InternetHttpHealthChecker,
    InternetHttpsHealthChecker, UptimeHealthChecker,
};
use crate::routes::health::models::{HealthCheckResponse, HealthResponse};
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::mysql;
use std::collections::HashMap;

#[get("")]
pub async fn get(
    database_pool: web::Data<mysql::MySqlPool>,
    http_client_pool: web::Data<reqwest::Client>,
) -> impl Responder {
    let uptime = UptimeHealthChecker::new();
    let database = DatabaseHealthChecker::new(database_pool.get_ref());
    let internet_http_health_checker = InternetHttpHealthChecker::new(http_client_pool.get_ref());
    let internet_https_health_checker = InternetHttpsHealthChecker::new(http_client_pool.get_ref());

    let x1 = uptime.check();
    let x2 = uptime.check();
    let x3 = database.check();
    let x4 = internet_http_health_checker.check();
    let x5 = internet_https_health_checker.check();
    let c = futures::future::join_all(vec![x1, x2, x3, x4, x5]).await;

    let health = Health::from(&c);

    let response = HealthResponse::from(&health);

    HttpResponse::Ok().json(response)
}
