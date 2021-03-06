use crate::health::checkers::{get_time, HealthCheckerAction};
use crate::health::HealthCheck;
use crate::health::HealthStatus;

pub(crate) struct UptimeHealthChecker;

impl UptimeHealthChecker {
    pub(crate) fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl HealthCheckerAction for UptimeHealthChecker {
    async fn check(&self) -> HealthCheck {
        HealthCheck {
            component_name: "uptime".to_owned(),
            component_id: None,
            component_type: Some("system".to_owned()),
            observed_value: None,
            observed_unit: None,
            status: Some(HealthStatus::Pass),
            affected_endpoints: None,
            time: get_time(),
            output: None,
            links: None,
            additional_keys: None,
        }
    }
}
