use super::models::GetResponse;
use actix_web::web::ServiceConfig;
use actix_web::{get, HttpResponse, Responder};
use sqlx::Row;

#[get("/status")]
pub async fn get(db_pool: actix_web::web::Data<sqlx::mysql::MySqlPool>) -> impl Responder {
    let database = sqlx::query("SELECT ? as Status")
        .bind("Healthy")
        .fetch_one(db_pool.get_ref())
        .await
        .unwrap()
        .get(0);

    HttpResponse::Ok().json(GetResponse {
        database,
        runtime: "Healthy".to_string(),
    })
}

pub fn configure(service_config: &mut ServiceConfig) {
    service_config.service(get);
}
