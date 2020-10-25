use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct Health {
    pub(crate) status: HealthStatus,
    pub(crate) version: Option<String>,
    pub(crate) release_id: Option<String>,
    pub(crate) notes: Option<Vec<String>>,
    pub(crate) output: Option<String>,
    pub(crate) checks: Option<Vec<HealthCheck>>,
    pub(crate) links: Option<HashMap<String, String>>,
    pub(crate) service_id: Option<String>,
    pub(crate) description: Option<String>,
}

#[derive(Clone)]
pub(crate) struct HealthCheck {
    pub(crate) component_name: String,
    pub(crate) component_id: Option<String>,
    pub(crate) component_type: Option<String>,
    pub(crate) observed_value: Option<String>,
    pub(crate) observed_unit: Option<String>,
    pub(crate) status: Option<HealthStatus>,
    pub(crate) affected_endpoints: Option<Vec<String>>,
    pub(crate) time: Option<String>,
    pub(crate) output: Option<String>,
    pub(crate) links: Option<HashMap<String, String>>,
    pub(crate) additional_keys: Option<HashMap<String, String>>,
}

#[derive(Clone, PartialEq)]
pub(crate) enum HealthStatus {
    Pass,
    Fail,
    Warn,
}

impl Health {
    pub(crate) fn from(checks: &[HealthCheck]) -> Self {
        let statuses = &checks
            .iter()
            .filter_map(|check| check.status.clone())
            .collect::<Vec<_>>();

        let status = if statuses.iter().any(|status| status.eq(&HealthStatus::Fail)) {
            HealthStatus::Fail
        } else if statuses.iter().any(|status| status.eq(&HealthStatus::Warn)) {
            HealthStatus::Warn
        } else {
            HealthStatus::Pass
        };

        Health {
            status,
            version: Some(env!("CARGO_PKG_VERSION_MAJOR").to_owned()),
            release_id: Some(env!("CARGO_PKG_VERSION").to_owned()),
            notes: None,
            output: None,
            checks: Some(checks.to_owned()),
            links: None,
            service_id: None,
            description: Some("health of rust-kata-001 service".to_owned()),
        }
    }
}
