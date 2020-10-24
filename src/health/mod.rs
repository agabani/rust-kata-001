pub mod database;
pub mod internet;
mod models;
pub mod runtime;

use async_trait::async_trait;
use futures::Future;
pub(crate) use models::{Health, HealthCheck, HealthStatus};

#[async_trait]
pub(crate) trait HealthChecker {
    async fn check(&self) -> HealthCheck;
}

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

pub(crate) async fn check_all<HC>(checks: &[impl Fn() -> HC]) -> Health
where
    HC: Future<Output = HealthCheck>,
{
    let health_checks: Vec<HealthCheck> =
        futures::future::join_all(checks.iter().map(|health_check| health_check())).await;

    Health {
        status: HealthStatus::Pass,
        version: None,
        release_id: None,
        notes: None,
        output: None,
        checks: None,
        links: None,
        service_id: None,
        description: None,
    }
}
