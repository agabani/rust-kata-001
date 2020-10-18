use actix_web::client::Client;

pub fn client() -> Client {
    Client::builder()
        .header(
            "User-Agent",
            "rust-kata-001 (https://github.com/agabani/rust-kata-001)",
        )
        .finish()
}
