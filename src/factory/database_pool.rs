pub(crate) async fn new(uri: &str) -> Result<sqlx::Pool<sqlx::mysql::MySql>, String> {
    let fn_name = "new";

    sqlx::mysql::MySqlPool::connect(uri).await.map_err(|error| {
        log::error!("{}: error={:?}", fn_name, error);
        format!("{}: error={:?}", fn_name, error)
    })
}
