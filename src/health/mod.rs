pub(crate) mod database_health_checker;
pub(crate) mod internet_http_health_checker;
pub(crate) mod internet_https_health_checker;
mod models;
pub(crate) mod uptime_health_checker;

pub(crate) use models::{Health, HealthCheck, HealthStatus};

#[async_trait::async_trait]
pub(crate) trait HealthChecker {
    async fn check(&self) -> HealthCheck;
}
