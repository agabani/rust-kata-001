use serde::Serialize;

#[derive(Serialize)]
pub struct GetResponse {
    pub database: String,
    pub runtime: String,
}
