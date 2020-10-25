pub(crate) mod internet_http_connectivity;
pub(crate) mod internet_https_connectivity;
pub(crate) mod mysql_connectivity;
pub(crate) mod uptime;

use crate::health::HealthCheck;
use crate::health::HealthStatus;
use chrono::{SecondsFormat, Utc};

#[async_trait::async_trait]
pub(crate) trait HealthCheckerAction {
    async fn check(&self) -> HealthCheck;
}

fn get_time() -> Option<String> {
    Some(Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true))
}

fn map_database_output(result: &Result<sqlx::mysql::MySqlRow, sqlx::Error>) -> Option<String> {
    match result {
        Ok(_) => None,
        Err(error) => Some(format!("{:?}", error)),
    }
}

fn map_database_status(
    result: &Result<sqlx::mysql::MySqlRow, sqlx::Error>,
) -> Option<HealthStatus> {
    match result {
        Ok(_) => Some(HealthStatus::Pass),
        Err(_) => Some(HealthStatus::Fail),
    }
}

fn map_internet_output(result: &Result<reqwest::Response, reqwest::Error>) -> Option<String> {
    match result {
        Ok(_) => None,
        Err(error) => Some(format!("{:?}", error)),
    }
}

fn map_internet_status(result: &Result<reqwest::Response, reqwest::Error>) -> Option<HealthStatus> {
    match result {
        Ok(_) => Some(HealthStatus::Pass),
        Err(_) => Some(HealthStatus::Fail),
    }
}
