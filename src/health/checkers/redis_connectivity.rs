use crate::health::checkers::{get_time, map_redis_output, map_redis_status, HealthCheckerAction};
use crate::health::HealthCheck;
use redis::aio::MultiplexedConnection;

pub(crate) struct RedisConnectivityHealthChecker<'a> {
    pool: &'a MultiplexedConnection,
}

impl<'a> RedisConnectivityHealthChecker<'a> {
    pub(crate) fn new(pool: &'a MultiplexedConnection) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl<'a> HealthCheckerAction for RedisConnectivityHealthChecker<'a> {
    async fn check(&self) -> HealthCheck {
        let mut connection = self.pool.clone();

        let value = redis::cmd("PING").query_async(&mut connection).await;

        let value = value.map_err(|error| format!("{:?}", error));

        HealthCheck {
            component_name: "redis:connectivity".to_string(),
            component_id: None,
            component_type: Some("datastore".to_owned()),
            observed_value: None,
            observed_unit: None,
            status: map_redis_status(&value),
            affected_endpoints: None,
            time: get_time(),
            output: map_redis_output(&value),
            links: None,
            additional_keys: None,
        }
    }
}
