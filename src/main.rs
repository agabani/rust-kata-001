use crate::factory::{database_pool, http_client};

mod factory;
mod graph;
mod status;

#[actix_web::main]
async fn main() -> Result<(), String> {
    let fn_name = "main";

    dotenv::dotenv().ok();
    env_logger::init();

    let server_address_host = std::env::var("HOST").expect("HOST is not set");
    let server_address_port = std::env::var("PORT").expect("PORT is not set");
    let database_pool_uri = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");

    let address = format!("{}:{}", server_address_host, server_address_port);

    let database_pool = database_pool::new(&database_pool_uri).await.map_err(|e| {
        log::error!("{}: error {:?}", fn_name, e);
        format!("{}: error: {:?}", fn_name, e)
    })?;

    let server = actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .data(http_client::new())
            .data(database_pool.clone())
            .configure(graph::configure)
            .configure(status::configure)
    })
    .bind(address);

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
