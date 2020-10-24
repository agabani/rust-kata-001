use crate::health::{Health, HealthCheck};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks: Option<HashMap<String, Vec<HealthCheckResponse>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct HealthCheckResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_value: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_unit: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub affected_endpoints: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_keys: Option<HashMap<String, String>>,
}

impl HealthResponse {
    pub(crate) fn from(health: &Health) -> Self {
        HealthResponse {
            status: "".to_string(),
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
}
