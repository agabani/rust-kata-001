use crate::health::{check_all, HealthChecker, UptimeHealthChecker};
use crate::routes::health::models::{HealthCheckResponse, HealthResponse};
use actix_web::{get, HttpResponse, Responder};
use std::collections::HashMap;

#[get("")]
pub async fn get() -> impl Responder {
    let uptime = UptimeHealthChecker::new();

    let checks = futures::future::join_all(vec![uptime.check()]).await;

    let u = || async { uptime.check().await };

    let vec1 = vec![u];
    let health = check_all(&vec1).await;

    let response = HealthResponse::from(&health);

    HttpResponse::Ok().json(response)
}
