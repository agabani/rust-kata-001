pub mod database;
pub mod internet;
mod models;
pub mod runtime;

use async_trait::async_trait;
use models::{HealthCheck, HealthStatus};

#[async_trait]
trait HealthChecker {
    async fn check(&self) -> HealthCheck;
}

struct UptimeHealthChecker;

impl UptimeHealthChecker {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl HealthChecker for UptimeHealthChecker {
    async fn check(&self) -> HealthCheck {
        HealthCheck {
            component_name: "uptime".to_owned(),
            component_id: None,
            component_type: None,
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
