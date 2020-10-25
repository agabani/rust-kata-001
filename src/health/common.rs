use crate::health::{HealthCheck, HealthStatus};

#[async_trait::async_trait]
pub(crate) trait HealthCheckerAction {
    async fn check(&self) -> HealthCheck;
}

pub(crate) fn map_database_status(
    result: &Result<sqlx::mysql::MySqlRow, sqlx::Error>,
) -> Option<HealthStatus> {
    match result {
        Ok(_) => Some(HealthStatus::Pass),
        Err(_) => Some(HealthStatus::Fail),
    }
}

pub(crate) fn map_internet_status(
    result: &Result<reqwest::Response, reqwest::Error>,
) -> Option<HealthStatus> {
    match result {
        Ok(_) => Some(HealthStatus::Pass),
        Err(_) => Some(HealthStatus::Fail),
    }
}
