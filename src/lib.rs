mod config;
mod factory;
mod graph;
mod status;

use actix_web::{App, HttpServer};
pub use config::Config;
use factory::{database_pool, http_client_pool};

pub async fn run(config: &Config) -> Result<(), String> {
    let fn_name = "run";

    let database_pool = database_pool::new(&config.database_url).await?;
    let http_client_pool = http_client_pool::new()?;

    HttpServer::new(move || {
        App::new()
            .data(database_pool.clone())
            .data(http_client_pool.clone())
            .configure(graph::configure)
            .configure(status::configure)
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
