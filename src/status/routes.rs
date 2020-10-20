use crate::status::{database, internet, runtime};
use actix_web::web::ServiceConfig;
use actix_web::{get, HttpResponse, Responder};
use std::collections::HashMap;

#[get("/status")]
pub async fn get(db_pool: actix_web::web::Data<sqlx::mysql::MySqlPool>) -> impl Responder {
    let database = database::database(db_pool.get_ref())
        .await
        .unwrap_or_else(error_to_model);

    let internet_http = internet::http().await.unwrap_or_else(error_to_model);
    let internet_https = internet::https().await.unwrap_or_else(error_to_model);
    let runtime = runtime::runtime().await.unwrap_or_else(error_to_model);

    let mut map = HashMap::new();

    map.entry("database").or_insert(database);
    map.entry("internet_http").or_insert(internet_http);
    map.entry("internet_https").or_insert(internet_https);
    map.entry("runtime").or_insert(runtime);

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
