use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct GetResponse {
    pub database: HashMap<String, String>,
    pub internet_http: HashMap<String, String>,
    pub internet_https: HashMap<String, String>,
    pub runtime: HashMap<String, String>,
}
