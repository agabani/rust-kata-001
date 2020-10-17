use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct GetResponse {
    pub database: HashMap<String, String>,
    pub runtime: HashMap<String, String>,
}
