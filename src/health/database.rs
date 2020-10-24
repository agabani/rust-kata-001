use sqlx::{MySqlPool, Row};
use std::collections::HashMap;

pub async fn database(pool: &MySqlPool) -> Result<HashMap<String, String>, String> {
    let fn_name = "database";

    let value = sqlx::query("SELECT ? as Status")
        .bind("healthy")
        .fetch_one(pool)
        .await
        .map_err(|e| {
            log::error!("{}: error {:?}", fn_name, e);
            format!("{:?}", e)
        })?
        .get(0);

    let mut map = HashMap::new();
    map.insert("status".to_owned(), value);
    Ok(map)
}
