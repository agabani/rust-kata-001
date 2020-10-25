use crate::api;
use crate::domain::Crate;
use crate::graph::{get_dependency, ApiGetOne, DatabaseGetOneBatch, DatabaseSaveOne};
use crate::persistence::Persistence;
use crate::routes::graph::models;
use actix_web::{get, web, HttpResponse, Responder};
use semver::Version;
use sqlx::mysql;
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

struct Dependency<'a> {
    http_client: &'a reqwest::Client,
    database_pool: &'a mysql::MySqlPool,
}

#[async_trait::async_trait]
impl<'a> ApiGetOne for Dependency<'a> {
    async fn execute(&self, name: String, version: &Version) -> Result<Crate, String> {
        let client = api::Client::new(self.http_client);
        client.get_crate(&name, &version).await
    }
}

#[async_trait::async_trait]
impl<'a> DatabaseGetOneBatch for Dependency<'a> {
    async fn execute(
        &self,
        crates: &[(String, Version)],
    ) -> Result<HashMap<(String, Version), Option<Crate>>, String> {
        let persistence = Persistence::new(self.database_pool);
        persistence.get_one_batch(crates).await
    }
}

#[async_trait::async_trait]
impl<'a> DatabaseSaveOne for Dependency<'a> {
    async fn execute(&self, c: &Crate) -> Result<(), String> {
        let persistence = Persistence::new(self.database_pool);
        persistence.save_one(c).await
    }
}
