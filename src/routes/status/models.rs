use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct GetResponse {
    pub database: HashMap<String, String>,
    pub internet_http: HashMap<String, String>,
    pub internet_https: HashMap<String, String>,
    pub runtime: HashMap<String, String>,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: Option<String>,
    pub release_id: Option<String>,
    pub notes: Option<Vec<String>>,
    pub output: Option<String>,
    pub checks: Option<HashMap<String, Vec<HealthCheckResponse>>>,
    pub links: Option<HashMap<String, String>>,
    pub service_id: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct HealthCheckResponse {
    pub component_id: Option<String>,
    pub component_type: Option<String>,
    pub observed_value: Option<String>,
    pub observed_unit: Option<String>,
    pub status: Option<String>,
    pub affected_endpoints: Option<Vec<String>>,
    pub time: Option<String>,
    pub output: Option<String>,
    pub links: Option<HashMap<String, String>>,
    pub additional_keys: Option<HashMap<String, String>>,
}
