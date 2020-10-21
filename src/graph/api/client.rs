use semver::Version;
use serde::Deserialize;

pub(crate) async fn dependencies(
    client: &reqwest::Client,
    name: String,
    version: String,
) -> Result<DependenciesApiDto, String> {
    let fn_name = "dependencies";

    let url = format!(
        "https://crates.io/api/v1/crates/{}/{}/dependencies",
        name, version
    );
    log::info!("{}: url={}", fn_name, url);

    let response = client.get(&url).send().await.map_err(|e| {
        log::error!("{}: send request error {:?}", fn_name, e);
        format!("{}: send request error: {:?}", fn_name, e)
    })?;
    log::info!("{}: status={}", fn_name, response.status());

    let dto = response.json::<DependenciesApiDto>().await.map_err(|e| {
        log::error!("{}: json payload error {:?}", fn_name, e);
        format!("{}: json payload error: {:?}", fn_name, e)
    })?;
    log::info!("{}: dto={:?}", fn_name, dto);

    Ok(dto)
}

pub(crate) async fn versions(
    client: &reqwest::Client,
    name: String,
) -> Result<VersionsApiDto, String> {
    let fn_name = "versions";

    let url = format!("https://crates.io/api/v1/crates/{}", name);
    log::info!("{}: url={}", fn_name, url);

    let response = client.get(&url).send().await.map_err(|e| {
        log::error!("{}: send request error {:?}", fn_name, e);
        format!("{}: send request error: {:?}", fn_name, e)
    })?;
    log::info!("{}: status={}", fn_name, response.status());

    let dto = response.json::<VersionsApiDto>().await.map_err(|e| {
        log::error!("{}: json payload error {:?}", fn_name, e);
        format!("{}: json payload error: {:?}", fn_name, e)
    })?;
    log::info!("{}: dto={:?}", fn_name, dto);

    Ok(dto)
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

#[derive(Debug, Deserialize)]
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

impl DependenciesApiDto {
    fn version(input: &str) -> Result<semver::Version, String> {
        use std::iter::FromIterator;

        let mut dots = 2;
        let mut chars = Vec::new();

        for char in input.trim_start_matches(|p| !char::is_numeric(p)).chars() {
            if char == '.' {
                dots -= 1;
            }

            if char == '*' && dots > 0 && chars.last() == Some(&'.') {
                chars.push('0');
            } else {
                chars.push(char);
            }
        }

        for _ in 0..dots {
            chars.push('.');
            chars.push('0');
        }

        match Version::parse(&String::from_iter(chars)) {
            Ok(t) => Ok(t),
            Err(e) => Err(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::factory::http_client;
    use crate::graph::api::client::{dependencies, versions};

    #[actix_rt::test]
    #[ignore]
    async fn integration_dependencies() -> Result<(), String> {
        let client = http_client::new()?;

        let result = dependencies(&client, "syn".to_owned(), "0.11.0".to_owned()).await?;

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
        let client = http_client::new()?;

        let result = versions(&client, "clippy".to_owned()).await?;

        assert!(result.errors.is_none());
        assert!(result.versions.is_some());

        let versions = result.versions.unwrap();

        assert!(versions.iter().any(|v| v.num == "0.0.2"));
        assert!(versions.iter().any(|v| v.num == "0.0.135"));
        assert!(versions.iter().any(|v| v.num == "0.0.302"));

        Ok(())
    }
}
