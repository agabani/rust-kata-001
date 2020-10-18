use crate::client::client;
use std::collections::HashMap;

pub async fn https() -> Result<HashMap<String, String>, String> {
    let fn_name = "https";

    let response = client()
        .get("https://httpbin.org/anything")
        .send()
        .await
        .map_err(|e| {
            log::error!("{}: send request error {:?}", fn_name, e);
            format!("{}: send request error: {:?}", fn_name, e)
        })?;

    Ok(status_code_to_model(response.status().as_u16()))
}

pub async fn http() -> Result<HashMap<String, String>, String> {
    let fn_name = "http";

    let response = client()
        .get("http://httpbin.org/anything")
        .send()
        .await
        .map_err(|e| {
            log::error!("{}: send request error {:?}", fn_name, e);
            format!("{}: send request error: {:?}", fn_name, e)
        })?;

    Ok(status_code_to_model(response.status().as_u16()))
}

fn status_code_to_model(status_code: u16) -> HashMap<String, String> {
    let mut map = HashMap::new();

    if status_code == 200 {
        map.insert("status".to_owned(), "healthy".to_owned());
    } else {
        map.insert("status".to_owned(), "unhealthy".to_owned());
        map.insert("status_code".to_owned(), status_code.to_string());
    }

    map
}
