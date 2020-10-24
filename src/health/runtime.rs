use std::collections::HashMap;

pub async fn runtime() -> Result<HashMap<String, String>, String> {
    let mut map = HashMap::new();
    map.insert("status".to_owned(), "healthy".to_owned());
    Ok(map)
}
