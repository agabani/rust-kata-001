use crate::health::checkers::HealthCheckerAction;
use crate::health::checkers::{get_time, map_internet_status};
use crate::health::HealthCheck;

pub(crate) struct InternetHttpsConnectivityHealthChecker<'a> {
    pool: &'a reqwest::Client,
}

impl<'a> InternetHttpsConnectivityHealthChecker<'a> {
    pub(crate) fn new(pool: &'a reqwest::Client) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl<'a> HealthCheckerAction for InternetHttpsConnectivityHealthChecker<'a> {
    async fn check(&self) -> HealthCheck {
        let response = self.pool.get("https://httpbin.org/anything").send().await;

        HealthCheck {
            component_name: "internet:https:connectivity".to_string(),
            component_id: None,
            component_type: Some("system".to_owned()),
            observed_value: None,
            observed_unit: None,
            status: map_internet_status(&response),
            affected_endpoints: None,
            time: get_time(),
            output: None,
            links: None,
            additional_keys: None,
        }
    }
}
