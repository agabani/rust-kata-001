mod database_health_checker;
mod internet_http_health_checker;
mod internet_https_health_checker;
mod models;
mod uptime_health_checker;

pub(crate) use database_health_checker::DatabaseHealthChecker;
pub(crate) use internet_http_health_checker::InternetHttpHealthChecker;
pub(crate) use internet_https_health_checker::InternetHttpsHealthChecker;
pub(crate) use models::{Health, HealthCheck, HealthStatus};
pub(crate) use uptime_health_checker::UptimeHealthChecker;

#[async_trait::async_trait]
pub(crate) trait HealthChecker {
    async fn check(&self) -> HealthCheck;
}
