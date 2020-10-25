mod checkers;
mod models;

use checkers::HealthCheckerAction;
use checkers::InternetHttpConnectivityHealthChecker;
use checkers::InternetHttpsConnectivityHealthChecker;
use checkers::MySqlConnectivityHealthChecker;
use checkers::RedisConnectivityHealthChecker;
use checkers::UptimeHealthChecker;

pub(crate) use models::{Health, HealthCheck, HealthStatus};

pub(crate) struct HealthChecker<'a> {
    internet_http_connectivity: InternetHttpConnectivityHealthChecker<'a>,
    internet_https_connectivity: InternetHttpsConnectivityHealthChecker<'a>,
    mysql_connectivity: MySqlConnectivityHealthChecker<'a>,
    redis_connectivity: RedisConnectivityHealthChecker<'a>,
    uptime: UptimeHealthChecker,
}

impl<'a> HealthChecker<'a> {
    pub(crate) fn new(
        database_pool: &'a sqlx::MySqlPool,
        http_client_pool: &'a reqwest::Client,
        redis_pool: &'a redis::aio::MultiplexedConnection,
    ) -> Self {
        Self {
            internet_http_connectivity: InternetHttpConnectivityHealthChecker::new(
                http_client_pool,
            ),
            internet_https_connectivity: InternetHttpsConnectivityHealthChecker::new(
                http_client_pool,
            ),
            mysql_connectivity: MySqlConnectivityHealthChecker::new(database_pool),
            redis_connectivity: RedisConnectivityHealthChecker::new(redis_pool),
            uptime: UptimeHealthChecker::new(),
        }
    }

    pub(crate) async fn check(&self) -> Health {
        let health_checks = futures::future::join_all(vec![
            self.internet_http_connectivity.check(),
            self.internet_https_connectivity.check(),
            self.mysql_connectivity.check(),
            self.redis_connectivity.check(),
            self.uptime.check(),
        ])
        .await;

        Health::from(&health_checks)
    }
}
