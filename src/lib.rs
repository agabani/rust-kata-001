mod config;
mod factory;
mod graph;
mod status;

pub use crate::config::Config;
use crate::factory::{database_pool, http_client};

pub async fn run(config: &Config) -> Result<(), String> {
    let fn_name = "run";

    let database_pool = database_pool::new(&config.database_url)
        .await
        .map_err(|e| {
            log::error!("{}: error {:?}", fn_name, e);
            format!("{}: error: {:?}", fn_name, e)
        })?;

    let http_client = http_client::new()?;

    let server = actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .data(http_client.clone())
            .data(database_pool.clone())
            .configure(graph::configure)
            .configure(status::configure)
    })
    .bind(&config.server_address);

    server
        .map_err(|e| {
            log::error!("{}: error {:?}", fn_name, e);
            format!("{}: error: {:?}", fn_name, e)
        })?
        .run()
        .await
        .map_err(|e| {
            log::error!("{}: error {:?}", fn_name, e);
            format!("{}: error: {:?}", fn_name, e)
        })
}
