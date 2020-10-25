mod common;
mod database_health_checker;
mod internet_http_health_checker;
mod internet_https_health_checker;
mod models;
mod uptime_health_checker;

pub(crate) use models::{Health, HealthCheck, HealthStatus};

use common::HealthCheckerAction;
use database_health_checker::DatabaseHealthChecker;
use internet_http_health_checker::InternetHttpHealthChecker;
use internet_https_health_checker::InternetHttpsHealthChecker;
use uptime_health_checker::UptimeHealthChecker;

pub(crate) struct HealthChecker<'a> {
    database: DatabaseHealthChecker<'a>,
    internet_http: InternetHttpHealthChecker<'a>,
    internet_https: InternetHttpsHealthChecker<'a>,
    uptime: UptimeHealthChecker,
}

impl<'a> HealthChecker<'a> {
    pub(crate) fn new(
        database_pool: &'a sqlx::MySqlPool,
        http_client_pool: &'a reqwest::Client,
    ) -> Self {
        Self {
            database: DatabaseHealthChecker::new(database_pool),
            internet_http: InternetHttpHealthChecker::new(http_client_pool),
            internet_https: InternetHttpsHealthChecker::new(http_client_pool),
            uptime: UptimeHealthChecker::new(),
        }
    }

    pub(crate) async fn check(&self) -> Health {
        let health_checks = futures::future::join_all(vec![
            self.database.check(),
            self.internet_http.check(),
            self.internet_https.check(),
            self.uptime.check(),
        ])
        .await;

        Health::from(&health_checks)
    }
}
