use crate::health::{Health, HealthCheck, HealthStatus};
use serde::Serialize;
use std::collections::HashMap;

pub(crate) enum HealthResponseStatus {
    Ok(HealthResponse),
    InternalServerError(HealthResponse),
}

#[derive(Serialize)]
pub struct HealthResponse {
    #[serde(rename = "status")]
    pub status: String,

    #[serde(skip_serializing_if = "Option::is_none", rename = "version")]
    pub version: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "releaseId")]
    pub release_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "notes")]
    pub notes: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "output")]
    pub output: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "serviceId")]
    pub service_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "description")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "checks")]
    pub checks: Option<HashMap<String, Vec<HealthCheckResponse>>>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "links")]
    pub links: Option<HashMap<String, String>>,
}

#[derive(Serialize)]
pub struct HealthCheckResponse {
    #[serde(skip_serializing_if = "Option::is_none", rename = "componentId")]
    pub component_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "componentType")]
    pub component_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "observedValue")]
    pub observed_value: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "observedUnit")]
    pub observed_unit: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "status")]
    pub status: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "affectedEndpoints")]
    pub affected_endpoints: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "time")]
    pub time: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "output")]
    pub output: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "links")]
    pub links: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "additionalKeys")]
    pub additional_keys: Option<HashMap<String, String>>,
}

impl HealthResponse {
    pub(crate) fn from(health: &Health) -> HealthResponseStatus {
        let response = HealthResponse {
            status: Self::map_status(&health.status),
            version: health.version.clone(),
            release_id: health.release_id.clone(),
            notes: health.notes.clone(),
            output: health.output.clone(),
            checks: Self::map_checks(&health.checks),
            links: health.links.clone(),
            service_id: health.service_id.clone(),
            description: health.description.clone(),
        };

        match health.status {
            HealthStatus::Pass => HealthResponseStatus::Ok(response),
            HealthStatus::Fail => HealthResponseStatus::InternalServerError(response),
            HealthStatus::Warn => HealthResponseStatus::Ok(response),
        }
    }

    fn map_status(status: &HealthStatus) -> String {
        match status {
            HealthStatus::Pass => "pass".to_owned(),
            HealthStatus::Fail => "fail".to_owned(),
            HealthStatus::Warn => "warn".to_owned(),
        }
    }

    fn map_checks(
        checks: &Option<Vec<HealthCheck>>,
    ) -> Option<HashMap<String, Vec<HealthCheckResponse>>> {
        if let Some(checks) = checks {
            let mut hashmap = HashMap::new();

            for check in checks {
                hashmap
                    .entry(check.component_name.to_owned())
                    .or_insert_with(Vec::new)
                    .push(HealthCheckResponse {
                        component_id: check.component_id.to_owned(),
                        component_type: check.component_type.to_owned(),
                        observed_value: check.observed_value.to_owned(),
                        observed_unit: check.observed_unit.to_owned(),
                        status: match &check.status {
                            Some(status) => Some(Self::map_status(status)),
                            None => None,
                        },
                        affected_endpoints: check.affected_endpoints.clone(),
                        time: check.time.clone(),
                        output: check.output.clone(),
                        links: check.links.clone(),
                        additional_keys: check.additional_keys.clone(),
                    });
            }

            Some(hashmap)
        } else {
            None
        }
    }
}
