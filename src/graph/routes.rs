use super::{api, data, models};
use crate::graph::domain::Crate;
use crate::graph::flow::{get_dependency, ApiGetOne, DatabaseGetOneBatch, DatabaseSaveOne};
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::mysql;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;

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
        // Some(version) => version,
        Some(version) => match semver::Version::parse(version) {
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

    // dependencies
    let dependency = Dependency {
        database_pool: database_pool.get_ref(),
        http_client: http_client.get_ref(),
    };

    // flow
    let result = get_dependency(
        &dependency,
        &dependency,
        &dependency,
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
            HttpResponse::InternalServerError().json(models::ErrorWebDto {
                status_code: 500,
                error_message: e,
            })
        }
    }
}

pub fn configure(service_config: &mut web::ServiceConfig) {
    service_config.service(web::scope("/graph").service(list));
}

struct Dependency<'a> {
    http_client: &'a reqwest::Client,
    database_pool: &'a mysql::MySqlPool,
}

#[async_trait::async_trait]
impl<'a> ApiGetOne for Dependency<'a> {
    async fn execute(&self, name: String, version: &semver::Version) -> Result<Crate, String> {
        let client = api::Client::new(self.http_client);
        client.get_crate(&name, &version).await
    }
}

#[async_trait::async_trait]
impl<'a> DatabaseGetOneBatch for Dependency<'a> {
    async fn execute(
        &self,
        crates: &[(String, semver::Version)],
    ) -> Result<HashMap<(String, semver::Version), Option<Crate>, RandomState>, String> {
        let database = data::Database::new(self.database_pool);
        database.get_one_batch(crates).await
    }
}

#[async_trait::async_trait]
impl<'a> DatabaseSaveOne for Dependency<'a> {
    async fn execute(&self, c: &Crate) -> Result<(), String> {
        let database = data::Database::new(self.database_pool);
        database.save_one(c).await
    }
}
