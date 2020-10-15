use super::{api, data, models};
use crate::graph::domain::Crate;
use crate::graph::flow::get_dependency;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::mysql;

#[get("")]
pub async fn list(
    database_pool: web::Data<mysql::MySqlPool>,
    query_parameters: web::Query<models::ListQueryParams>,
) -> impl Responder {
    // request
    let name = match &query_parameters.name {
        Some(name) => name,
        None => return HttpResponse::BadRequest().json(models::ErrorWebDto { status_code: 400 }),
    };

    let version = match &query_parameters.version {
        Some(version) => version,
        None => return HttpResponse::BadRequest().json(models::ErrorWebDto { status_code: 400 }),
    };

    // dependencies
    let db_get_one = |name: String, version: String| async {
        data::CrateDataDto::get_one(database_pool.get_ref(), name, version).await
    };

    let db_save_one =
        |c: Crate| async { data::CrateDataDto::save_one(database_pool.get_ref(), c).await };

    let api_get_one =
        |name: String, version: String| async { api::client::dependencies(name, version).await };

    let api_get_versions = |name: String| async { api::client::versions(name).await };

    // flow
    let result = get_dependency(
        db_get_one,
        db_save_one,
        api_get_one,
        api_get_versions,
        name.to_owned(),
        version.to_owned(),
    )
    .await;

    // response
    match result {
        Ok(c) => HttpResponse::Ok().json(
            c.iter()
                .map(models::CrateWebDto::transform)
                .collect::<Vec<_>>(),
        ),
        Err(e) => {
            log::error!("{}", e);
            HttpResponse::InternalServerError().json(e)
        }
    }
}

pub fn configure(service_config: &mut web::ServiceConfig) {
    service_config.service(web::scope("/graph").service(list));
}
