use crate::graph::domain::{Crate, CrateDependency};
use actix_web::client::Client;
use semver::Version;
use serde::Deserialize;

fn client() -> Client {
    Client::builder()
        .header(
            "User-Agent",
            "rust-kata-001 (https://github.com/agabani/rust-kata-001)",
        )
        .finish()
}

pub async fn dependencies(name: String, version: String) -> Result<Crate, String> {
    let fn_name = "dependencies";

    let url = format!(
        "https://crates.io/api/v1/crates/{}/{}/dependencies",
        name, version
    );
    log::info!("{}: url={}", fn_name, url);

    let mut response = client().get(url).send().await.map_err(|e| {
        log::error!("{}: send request error {:?}", fn_name, e);
        format!("{}: send request error: {:?}", fn_name, e)
    })?;
    log::info!("{}: status={}", fn_name, response.status());

    let dto = response.json::<DependenciesApiDto>().await.map_err(|e| {
        log::error!("{}: json payload error {:?}", fn_name, e);
        format!("{}: json payload error: {:?}", fn_name, e)
    })?;
    log::info!("{}: dto={:?}", fn_name, dto);

    Ok(DependenciesApiDto::transform(&name, &version, &dto))
}

pub async fn versions(name: String) -> Result<Vec<Version>, String> {
    let fn_name = "versions";

    let url = format!("https://crates.io/api/v1/crates/{}", name);
    log::info!("{}: url={}", fn_name, url);

    let mut response = client().get(url).send().await.map_err(|e| {
        log::error!("{}: send request error {:?}", fn_name, e);
        format!("{}: send request error: {:?}", fn_name, e)
    })?;
    log::info!("{}: status={}", fn_name, response.status());

    let dto = response
        .json::<VersionsApiDto>()
        .limit(20_000_000)
        .await
        .map_err(|e| {
            log::error!("{}: json payload error {:?}", fn_name, e);
            format!("{}: json payload error: {:?}", fn_name, e)
        })?;
    log::info!("{}: dto={:?}", fn_name, dto);

    Ok(dto
        .versions
        .iter()
        .map(|v| Version::parse(&v.num).unwrap())
        .collect())
}

#[derive(Debug, Deserialize)]
struct DependenciesApiDto {
    dependencies: Vec<DependencyApiDto>,
}

#[derive(Debug, Deserialize)]
struct DependencyApiDto {
    id: i32,
    version_id: i32,
    crate_id: String,
    req: String,
    optional: bool,
    default_features: bool,
    features: Vec<String>,
    target: Option<String>,
    kind: String,
    downloads: i32,
}

#[derive(Debug, Deserialize)]
struct VersionsApiDto {
    versions: Vec<VersionApiDto>,
}

#[derive(Debug, Deserialize)]
struct VersionApiDto {
    num: String,
}

impl DependenciesApiDto {
    fn transform(name: &str, version: &str, dependencies: &DependenciesApiDto) -> Crate {
        Crate {
            name: name.to_owned(),
            version: DependenciesApiDto::version(version).unwrap(),
            dependency: dependencies
                .dependencies
                .iter()
                .filter(|d| d.kind == "normal")
                .map(|d| CrateDependency {
                    name: d.crate_id.to_owned(),
                    version: DependenciesApiDto::version(&d.req).unwrap(),
                })
                .collect(),
        }
    }

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
    use super::*;

    #[test]
    fn transform() {
        let name = "name";
        let version = "version";
        let dependencies = DependenciesApiDto {
            dependencies: vec![
                DependencyApiDto {
                    id: 1,
                    version_id: 1,
                    crate_id: "sub name 1".to_owned(),
                    req: "sub version 1".to_owned(),
                    optional: false,
                    default_features: false,
                    features: vec![],
                    target: Some("target".to_owned()),
                    kind: "kind".to_owned(),
                    downloads: 1,
                },
                DependencyApiDto {
                    id: 1,
                    version_id: 1,
                    crate_id: "sub name 2".to_owned(),
                    req: "sub version 2".to_owned(),
                    optional: false,
                    default_features: false,
                    features: vec![],
                    target: None,
                    kind: "kind".to_owned(),
                    downloads: 1,
                },
            ],
        };

        let expected = Crate {
            name: "name".to_owned(),
            version: semver::Version::parse("version").unwrap(),
            dependency: vec![
                CrateDependency {
                    name: "sub name 1".to_owned(),
                    version: semver::Version::parse("sub version 1").unwrap(),
                },
                CrateDependency {
                    name: "sub name 2".to_owned(),
                    version: semver::Version::parse("sub version 2").unwrap(),
                },
            ],
        };

        let actual = DependenciesApiDto::transform(name, version, &dependencies);

        assert_eq!(actual, expected);
    }
}
