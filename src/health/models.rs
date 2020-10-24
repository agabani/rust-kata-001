use std::collections::HashMap;

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

pub(crate) enum HealthStatus {
    Pass,
    Fail,
    Warn,
}
