use super::{HealthCheck, HealthCheckerAction, HealthStatus};
use reqwest::Response;

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

#[async_trait::async_trait]
impl<'a> HealthCheckerAction for InternetHttpsHealthChecker<'a> {
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
