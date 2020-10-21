use crate::status::{database, internet, runtime};
use actix_web::web::ServiceConfig;
use actix_web::{get, HttpResponse, Responder};
use std::collections::HashMap;

#[get("/status")]
pub async fn get(
    http_client: actix_web::web::Data<reqwest::Client>,
    database_pool: actix_web::web::Data<sqlx::mysql::MySqlPool>,
) -> impl Responder {
    let (database, internet_http, internet_https, runtime) = futures::join!(
        database::database(database_pool.get_ref()),
        internet::http(http_client.get_ref()),
        internet::https(http_client.get_ref()),
        runtime::runtime()
    );

    let mut map = HashMap::new();
    map.entry("database")
        .or_insert_with(|| database.unwrap_or_else(error_to_model));
    map.entry("internet_http")
        .or_insert_with(|| internet_http.unwrap_or_else(error_to_model));
    map.entry("internet_https")
        .or_insert_with(|| internet_https.unwrap_or_else(error_to_model));
    map.entry("runtime")
        .or_insert_with(|| runtime.unwrap_or_else(error_to_model));

    if map.iter().all(|(&k, _)| k == "healthy") {
        HttpResponse::Ok().json(map)
    } else {
        HttpResponse::InternalServerError().json(map)
    }
}

fn error_to_model(reason: String) -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("status".to_owned(), "unhealthy".to_owned());
    map.insert("reason".to_owned(), reason);
    map
}

pub fn configure(service_config: &mut ServiceConfig) {
    service_config.service(get);
}
