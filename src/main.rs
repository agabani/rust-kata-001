use rust_kata_001::Config;
use std::env;

#[actix_web::main]
async fn main() -> Result<(), String> {
    dotenv::dotenv().ok();
    env_logger::init();

    let database_url = resolve_parameter("DATABASE_URL")?;
    let server_address_host = resolve_parameter("SERVER_ADDRESS_HOST")?;
    let server_address_port = resolve_parameter("SERVER_ADDRESS_PORT")?;

    let config = Config::new(&database_url, &server_address_host, &server_address_port);

    rust_kata_001::run(&config).await
}

fn resolve_parameter(key: &str) -> Result<String, String> {
    let fn_name = "resolve_parameter";

    env::var(key).map_err(|error| {
        log::error!("{}: key={:?} error={:?}", fn_name, key, error);
        format!("{}: key={:?} error={:?}", fn_name, key, error)
    })
}
