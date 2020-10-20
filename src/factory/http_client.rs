pub fn new() -> actix_web::client::Client {
    actix_web::client::Client::builder()
        .header(
            "User-Agent",
            "rust-kata-001 (https://github.com/agabani/rust-kata-001)",
        )
        .finish()
}
