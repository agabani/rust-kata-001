use serde::Deserialize;

pub struct CratesIoClient<'a> {
    http_client: &'a reqwest::Client,
}

impl<'a> CratesIoClient<'a> {
    pub fn new(client: &'a reqwest::Client) -> CratesIoClient<'a> {
        CratesIoClient {
            http_client: client,
        }
    }

    /// Gets the dependencies of a crate.
    pub(crate) async fn dependencies(
        &self,
        name: &str,
        version: &str,
    ) -> Result<DependenciesApiDto, String> {
        let url = format!(
            "https://crates.io/api/v1/crates/{}/{}/dependencies",
            name, version
        );

        let dto = self.get("dependencies", &url).await?;

        Ok(dto)
    }

    /// Gets the versions of a crate.
    pub(crate) async fn versions(&self, name: &str) -> Result<VersionsApiDto, String> {
        let url = format!("https://crates.io/api/v1/crates/{}", name);

        let dto = self.get("versions", &url).await?;

        Ok(dto)
    }

    async fn get<T: std::fmt::Debug + serde::de::DeserializeOwned>(
        &self,
        fn_name: &str,
        url: &str,
    ) -> Result<T, String> {
        log::info!("{}: url={}", fn_name, url);

        let response = self.http_client.get(url).send().await.map_err(|e| {
            log::error!("{}: send request error {:?}", fn_name, e);
            format!("{}: send request error: {:?}", fn_name, e)
        })?;
        log::info!("{}: status={}", fn_name, response.status());

        let dto = response.json::<T>().await.map_err(|e| {
            log::error!("{}: json payload error {:?}", fn_name, e);
            format!("{}: json payload error: {:?}", fn_name, e)
        })?;
        log::info!("{}: dto={:?}", fn_name, dto);

        Ok(dto)
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct ErrorApiDto {
    pub(crate) detail: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DependenciesApiDto {
    pub(crate) dependencies: Option<Vec<DependencyApiDto>>,
    pub(crate) errors: Option<Vec<ErrorApiDto>>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct DependencyApiDto {
    pub(crate) id: i32,
    pub(crate) version_id: i32,
    pub(crate) crate_id: String,
    pub(crate) req: String,
    pub(crate) optional: bool,
    pub(crate) default_features: bool,
    pub(crate) features: Vec<String>,
    pub(crate) target: Option<String>,
    pub(crate) kind: String,
    pub(crate) downloads: i32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct VersionsApiDto {
    pub(crate) versions: Option<Vec<VersionApiDto>>,
    pub(crate) errors: Option<Vec<ErrorApiDto>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct VersionApiDto {
    pub(crate) num: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::http_client_pool;

    #[actix_rt::test]
    #[ignore]
    async fn integration_dependencies() -> Result<(), String> {
        let client = http_client_pool::new()?;
        let client = CratesIoClient::new(&client);

        let result = client.dependencies("syn", "0.11.0").await?;

        assert!(result.errors.is_none());
        assert!(result.dependencies.is_some());

        let dependencies = result.dependencies.unwrap();

        assert_eq!(dependencies.len(), 8);
        assert!(dependencies
            .iter()
            .any(|d| d.crate_id == "syntex_pos" && d.req == "^0.52.0"));
        assert!(dependencies
            .iter()
            .any(|d| d.crate_id == "syntex_syntax" && d.req == "^0.52.0"));
        assert!(dependencies
            .iter()
            .any(|d| d.crate_id == "tempdir" && d.req == "^0.3.5"));
        assert!(dependencies
            .iter()
            .any(|d| d.crate_id == "time" && d.req == "^0.1.35"));
        assert!(dependencies
            .iter()
            .any(|d| d.crate_id == "walkdir" && d.req == "^1.0.1"));
        assert!(dependencies
            .iter()
            .any(|d| d.crate_id == "clippy" && d.req == "0.*"));
        assert!(dependencies
            .iter()
            .any(|d| d.crate_id == "quote" && d.req == "^0.3.0"));
        assert!(dependencies
            .iter()
            .any(|d| d.crate_id == "unicode-xid" && d.req == "^0.0.3"));

        Ok(())
    }

    #[actix_rt::test]
    #[ignore]
    async fn integration_versions() -> Result<(), String> {
        let client = http_client_pool::new()?;
        let client = CratesIoClient::new(&client);

        let result = client.versions("clippy").await?;

        assert!(result.errors.is_none());
        assert!(result.versions.is_some());

        let versions = result.versions.unwrap();

        assert!(versions.iter().any(|v| v.num == "0.0.2"));
        assert!(versions.iter().any(|v| v.num == "0.0.135"));
        assert!(versions.iter().any(|v| v.num == "0.0.302"));

        Ok(())
    }
}
