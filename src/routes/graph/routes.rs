use crate::data::Data;
use crate::routes::graph::models;
use actix_web::{get, web, HttpResponse, Responder};
use semver::Version;
use sqlx::mysql;

#[get("")]
pub async fn list(
    database_pool: web::Data<mysql::MySqlPool>,
    http_client: web::Data<reqwest::Client>,
    query_parameters: web::Query<models::ListQueryParams>,
) -> impl Responder {
    // request
    let name = match &query_parameters.name {
        Some(name) => name,
        None => {
            return HttpResponse::BadRequest().json(models::ErrorWebDto {
                status_code: 400,
                error_message: "name is required".to_owned(),
            })
        }
    };

    let version = match &query_parameters.version {
        Some(version) => match Version::parse(version) {
            Ok(version) => version,
            Err(e) => {
                return HttpResponse::BadRequest().json(models::ErrorWebDto {
                    status_code: 400,
                    error_message: format!("version invalid: {:?}", e),
                })
            }
        },
        None => {
            return HttpResponse::BadRequest().json(models::ErrorWebDto {
                status_code: 400,
                error_message: "version is required".to_owned(),
            })
        }
    };

    // data
    let result = Data::new(database_pool.get_ref(), http_client.get_ref())
        .get_dependency_graph(name.to_owned(), version.to_owned())
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
            HttpResponse::InternalServerError().json(models::ErrorWebDto {
                status_code: 500,
                error_message: e,
            })
        }
    }
}
