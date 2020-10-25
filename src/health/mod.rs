mod checkers;
mod models;

use checkers::internet_http_connectivity::InternetHttpConnectivityHealthChecker;
use checkers::internet_https_connectivity::InternetHttpsConnectivityHealthChecker;
use checkers::mysql_connectivity::MySqlConnectivityHealthChecker;
use checkers::uptime::UptimeHealthChecker;
use checkers::HealthCheckerAction;

pub(crate) use models::{Health, HealthCheck, HealthStatus};

pub(crate) struct HealthChecker<'a> {
    internet_http_connectivity: InternetHttpConnectivityHealthChecker<'a>,
    internet_https_connectivity: InternetHttpsConnectivityHealthChecker<'a>,
    mysql_connectivity: MySqlConnectivityHealthChecker<'a>,
    uptime: UptimeHealthChecker,
}

impl<'a> HealthChecker<'a> {
    pub(crate) fn new(
        database_pool: &'a sqlx::MySqlPool,
        http_client_pool: &'a reqwest::Client,
    ) -> Self {
        Self {
            internet_http_connectivity: InternetHttpConnectivityHealthChecker::new(
                http_client_pool,
            ),
            internet_https_connectivity: InternetHttpsConnectivityHealthChecker::new(
                http_client_pool,
            ),
            mysql_connectivity: MySqlConnectivityHealthChecker::new(database_pool),
            uptime: UptimeHealthChecker::new(),
        }
    }

    pub(crate) async fn check(&self) -> Health {
        let health_checks = futures::future::join_all(vec![
            self.internet_http_connectivity.check(),
            self.internet_https_connectivity.check(),
            self.mysql_connectivity.check(),
            self.uptime.check(),
        ])
        .await;

        Health::from(&health_checks)
    }
}
