pub async fn new(uri: &str) -> Result<sqlx::Pool<sqlx::mysql::MySql>, String> {
    let fn_name = "new";

    sqlx::mysql::MySqlPool::connect(uri).await.map_err(|e| {
        log::error!("{}: error {:?}", fn_name, e);
        format!("{}: error: {:?}", fn_name, e)
    })
}
