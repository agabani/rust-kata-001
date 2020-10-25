#[derive(Debug)]
pub struct Config {
    pub(crate) mysql_url: String,
    pub(crate) redis_url: String,
    pub(crate) server_address: String,
}

impl Config {
    pub fn new(
        mysql_url: &str,
        redis_url: &str,
        server_address_host: &str,
        server_address_port: &str,
    ) -> Self {
        let config = Self {
            mysql_url: mysql_url.to_owned(),
            redis_url: redis_url.to_owned(),
            server_address: format!("{}:{}", server_address_host, server_address_port),
        };

        log::debug!("{:?}", config);

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let config = Config::new(
            "my mysql url",
            "my redis url",
            "my server address host",
            "my server address port",
        );

        assert_eq!(config.mysql_url, "my mysql url");
        assert_eq!(config.redis_url, "my redis url");
        assert_eq!(
            config.server_address,
            "my server address host:my server address port"
        );
    }
}
