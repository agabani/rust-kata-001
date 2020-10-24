use crate::health::{HealthCheck, HealthChecker, HealthStatus};

pub(crate) struct UptimeHealthChecker;

impl UptimeHealthChecker {
    pub(crate) fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
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
