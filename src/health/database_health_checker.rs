use super::HealthCheck;
use crate::health::common::{map_database_status, HealthCheckerAction};

pub(crate) struct DatabaseHealthChecker<'a> {
    pool: &'a sqlx::MySqlPool,
}

impl<'a> DatabaseHealthChecker<'a> {
    pub(crate) fn new(pool: &'a sqlx::MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl<'a> HealthCheckerAction for DatabaseHealthChecker<'a> {
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
            status: map_database_status(&value),
            affected_endpoints: None,
            time: None,
            output: None,
            links: None,
            additional_keys: None,
        }
    }
}
