mod internet_http_connectivity;
mod internet_https_connectivity;
mod mysql_connectivity;
mod redis_connectivity;
mod uptime;

use crate::health::HealthCheck;
use crate::health::HealthStatus;
use chrono::{SecondsFormat, Utc};

pub(crate) use internet_http_connectivity::InternetHttpConnectivityHealthChecker;
pub(crate) use internet_https_connectivity::InternetHttpsConnectivityHealthChecker;
pub(crate) use mysql_connectivity::MySqlConnectivityHealthChecker;
pub(crate) use redis_connectivity::RedisConnectivityHealthChecker;
pub(crate) use uptime::UptimeHealthChecker;

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

fn map_redis_output(result: &Result<(), String>) -> Option<String> {
    match result {
        Ok(_) => None,
        Err(error) => Some(error.to_owned()),
    }
}

fn map_redis_status(result: &Result<(), String>) -> Option<HealthStatus> {
    match result {
        Ok(_) => Some(HealthStatus::Pass),
        Err(_) => Some(HealthStatus::Fail),
    }
}
