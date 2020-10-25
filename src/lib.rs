mod api;
mod config;
mod data;
mod domain;
mod factory;
mod health;
mod persistence;
mod routes;

use crate::factory::database_pool;
use crate::factory::http_client_pool;
use crate::factory::redis_pool;
use actix_web::{App, HttpServer};

pub use config::Config;

pub async fn run(config: &Config) -> Result<(), String> {
    let fn_name = "run";

    let database_pool = database_pool::new(&config.mysql_url).await?;
    let http_client_pool = http_client_pool::new()?;
    let redis_pool = redis_pool::new(&config.redis_url).await?;

    HttpServer::new(move || {
        App::new()
            .data(database_pool.clone())
            .data(http_client_pool.clone())
            .data(redis_pool.clone())
            .configure(routes::configure)
    })
    .bind(&config.server_address)
    .map_err(|error| {
        log::error!("{}: error={:?}", fn_name, error);
        format!("{}: error={:?}", fn_name, error)
    })?
    .run()
    .await
    .map_err(|error| {
        log::error!("{}: error={:?}", fn_name, error);
        format!("{}: error={:?}", fn_name, error)
    })
}
