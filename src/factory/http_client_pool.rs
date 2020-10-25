pub(crate) fn new() -> Result<reqwest::Client, String> {
    let fn_name = "new";

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static(
            "rust-kata-001 (https://github.com/agabani/rust-kata-001)",
        ),
    );

    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|error| {
            log::error!("{}: error={:?}", fn_name, error);
            format!("{}: error={:?}", fn_name, error)
        })
}
