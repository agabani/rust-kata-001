pub mod database;
pub mod internet;
mod models;
pub mod runtime;

use async_trait::async_trait;
use futures::Future;
pub(crate) use models::{Health, HealthCheck, HealthStatus};
use reqwest::Response;
use sqlx::mysql::MySqlRow;
use sqlx::{mysql, Error};

#[async_trait]
pub(crate) trait HealthChecker {
    async fn check(&self) -> HealthCheck;
}

// uptime
pub(crate) struct UptimeHealthChecker;

impl UptimeHealthChecker {
    pub(crate) fn new() -> Self {
        Self
    }
}

#[async_trait]
impl HealthChecker for UptimeHealthChecker {
    async fn check(&self) -> HealthCheck {
        HealthCheck {
            component_name: "uptime".to_owned(),
            component_id: None,
            component_type: Some("system".to_owned()),
            observed_value: None,
            observed_unit: None,
            status: Some(HealthStatus::Pass),
            affected_endpoints: None,
            time: None,
            output: None,
            links: None,
            additional_keys: None,
        }
    }
}

// database
pub(crate) struct DatabaseHealthChecker<'a> {
    pool: &'a mysql::MySqlPool,
}

impl<'a> DatabaseHealthChecker<'a> {
    pub(crate) fn new(pool: &'a mysql::MySqlPool) -> Self {
        Self { pool }
    }

    fn map_status(result: &Result<MySqlRow, Error>) -> Option<HealthStatus> {
        match result {
            Ok(_) => Some(HealthStatus::Pass),
            Err(_) => Some(HealthStatus::Fail),
        }
    }
}

#[async_trait]
impl<'a> HealthChecker for DatabaseHealthChecker<'a> {
    async fn check(&self) -> HealthCheck {
        let value = sqlx::query("SELECT ? as Status")
            .bind("healthy")
            .fetch_one(self.pool)
            .await;

        HealthCheck {
            component_name: "mysql:connectivity".to_string(),
            component_id: None,
            component_type: Some("datastore".to_owned()),
            observed_value: None,
            observed_unit: None,
            status: Self::map_status(&value),
            affected_endpoints: None,
            time: None,
            output: None,
            links: None,
            additional_keys: None,
        }
    }
}

// internet http
pub(crate) struct InternetHttpHealthChecker<'a> {
    pool: &'a reqwest::Client,
}

impl<'a> InternetHttpHealthChecker<'a> {
    pub(crate) fn new(pool: &'a reqwest::Client) -> Self {
        Self { pool }
    }

    fn map_status(result: &Result<Response, reqwest::Error>) -> Option<HealthStatus> {
        match result {
            Ok(_) => Some(HealthStatus::Pass),
            Err(_) => Some(HealthStatus::Fail),
        }
    }
}

#[async_trait]
impl<'a> HealthChecker for InternetHttpHealthChecker<'a> {
    async fn check(&self) -> HealthCheck {
        let response = self.pool.get("http://httpbin.org/anything").send().await;

        HealthCheck {
            component_name: "internet:http:connectivity".to_string(),
            component_id: None,
            component_type: Some("system".to_owned()),
            observed_value: None,
            observed_unit: None,
            status: Self::map_status(&response),
            affected_endpoints: None,
            time: None,
            output: None,
            links: None,
            additional_keys: None,
        }
    }
}

// internet http
pub(crate) struct InternetHttpsHealthChecker<'a> {
    pool: &'a reqwest::Client,
}

impl<'a> InternetHttpsHealthChecker<'a> {
    pub(crate) fn new(pool: &'a reqwest::Client) -> Self {
        Self { pool }
    }

    fn map_status(result: &Result<Response, reqwest::Error>) -> Option<HealthStatus> {
        match result {
            Ok(_) => Some(HealthStatus::Pass),
            Err(_) => Some(HealthStatus::Fail),
        }
    }
}

#[async_trait]
impl<'a> HealthChecker for InternetHttpsHealthChecker<'a> {
    async fn check(&self) -> HealthCheck {
        let response = self.pool.get("https://httpbin.org/anything").send().await;

        HealthCheck {
            component_name: "internet:https:connectivity".to_string(),
            component_id: None,
            component_type: Some("system".to_owned()),
            observed_value: None,
            observed_unit: None,
            status: Self::map_status(&response),
            affected_endpoints: None,
            time: None,
            output: None,
            links: None,
            additional_keys: None,
        }
    }
}
