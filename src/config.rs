#[derive(Debug)]
pub struct Config {
    pub(crate) database_url: String,
    pub(crate) server_address: String,
}

impl Config {
    pub fn new(database_url: &str, server_address_host: &str, server_address_port: &str) -> Self {
        let config = Self {
            database_url: database_url.to_owned(),
            server_address: format!("{}:{}", server_address_host, server_address_port),
        };

        log::debug!("{:?}", config);

        config
    }
}
