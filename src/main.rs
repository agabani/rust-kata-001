mod example;
mod graph;
mod status;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let address = format!(
        "{}:{}",
        std::env::var("HOST").expect("HOST is not set"),
        std::env::var("PORT").expect("PORT is not set")
    );

    let pool = sqlx::mysql::MySqlPool::connect(
        &std::env::var("DATABASE_URL").expect("DATABASE_URL is not set"),
    )
    .await
    .expect("");

    let server = actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .data(pool.clone())
            .configure(example::configure)
            .configure(graph::configure)
            .configure(status::configure)
    })
    .bind(address);

    server?.run().await
}
