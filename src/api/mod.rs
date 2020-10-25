mod crates_io_client;

use crate::api::crates_io_client::{CratesIoClient, DependencyApiDto};
use crate::domain::{Crate, CrateDependency};
use semver::Version;
use std::collections::HashMap;

pub(crate) struct Api<'a> {
    crates_io_client: CratesIoClient<'a>,
}

impl<'a> Api<'a> {
    pub(crate) fn new(client: &'a reqwest::Client) -> Api<'a> {
        Api {
            crates_io_client: CratesIoClient::new(client),
        }
    }

    /// Gets a crate.
    pub(crate) async fn get_crate(&self, name: &str, version: &Version) -> Result<Crate, String> {
        let fn_name = "get_crate";

        let dto = self
            .crates_io_client
            .dependencies(name, &version.to_string())
            .await?;

        if let Some(e) = dto.errors {
            log::error!("{}: crates.io client error {:?}", fn_name, e);
            return Err(format!("{}: crates.io client error: {:?}", fn_name, e));
        }

        let dependencies = dto.dependencies.ok_or_else(|| {
            log::error!("{}: crates.io contract violation", fn_name);
            format!("{}: crates.io contract violation", fn_name)
        })?;

        let crate_dependencies =
            futures::future::join_all(dependencies.iter().filter_map(|dependency| {
                if dependency.kind == "normal" {
                    Some(self.convert_or_best_guess(dependency))
                } else {
                    None
                }
            }))
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        let mut results = HashMap::new();

        for crate_dependency in crate_dependencies.iter() {
            results
                .entry((&crate_dependency.name, &crate_dependency.version))
                .or_insert(CrateDependency {
                    name: crate_dependency.name.to_owned(),
                    version: crate_dependency.version.to_owned(),
                });
        }

        Ok(Crate {
            name: name.to_owned(),
            version: version.to_owned(),
            dependency: results.into_iter().map(|e| e.1).collect(),
        })
    }

    async fn convert_or_best_guess(
        &self,
        dependency: &DependencyApiDto,
    ) -> Result<CrateDependency, String> {
        if let Some(crate_dependency) = Self::convert(dependency)? {
            Ok(crate_dependency)
        } else {
            self.best_guess(dependency).await
        }
    }

    fn convert(dependency: &DependencyApiDto) -> Result<Option<CrateDependency>, String> {
        let fn_name = "convert";

        if let Some(version) = Self::sanitise_version(&dependency.req) {
            let version = Version::parse(&version).map_err(|e| {
                log::error!("{}: sem ver error {:?}", fn_name, e);
                format!("{}: sem ver error: {:?}", fn_name, e)
            })?;

            Ok(Some(CrateDependency {
                name: dependency.crate_id.to_owned(),
                version,
            }))
        } else {
            Ok(None)
        }
    }

    async fn best_guess(&self, dependency: &DependencyApiDto) -> Result<CrateDependency, String> {
        let fn_name = "best_guess";

        let version_reqs = Self::parse_requirements(&dependency.req)?;

        let dto = self.crates_io_client.versions(&dependency.crate_id).await?;

        if let Some(e) = dto.errors {
            log::error!("{}: crates.io client error {:?}", fn_name, e);
            return Err(format!("{}: crates.io client error: {:?}", fn_name, e));
        }

        let versions = dto
            .versions
            .ok_or_else(|| {
                log::error!("{}: crates.io contract violation", fn_name);
                format!("{}: crates.io contract violation", fn_name)
            })?
            .iter()
            .map(|version| Version::parse(&version.num))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                log::error!("{}: sem ver error {:?}", fn_name, e);
                format!("{}: sem ver error: {:?}", fn_name, e)
            })?;

        let mut matching_versions = versions
            .iter()
            .filter(|&v| version_reqs.iter().all(|vr| vr.matches(v)))
            .collect::<Vec<_>>();

        matching_versions.sort();
        let best_version = matching_versions.swap_remove(0);

        Ok(CrateDependency {
            name: dependency.crate_id.to_owned(),
            version: best_version.to_owned(),
        })
    }

    fn parse_requirements(requirements: &str) -> Result<Vec<semver::VersionReq>, String> {
        let fn_name = "parse_requirements";

        requirements
            .split(',')
            .into_iter()
            .map(|f| semver::VersionReq::parse(f.trim()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                log::error!("{}: req parse error {:?}", fn_name, e);
                format!("{}: req parse error: {:?}", fn_name, e)
            })
    }

    fn sanitise_version(version: &str) -> Option<String> {
        // check for multi requirements
        if version.split(',').into_iter().count() > 1 {
            return None;
        }

        // check for numerical major.minor.build-
        let components = version
            .split(|p| p == '.' || p == '-')
            .into_iter()
            .collect::<Vec<_>>();

        if components.len() < 3
            || components
                .iter()
                .take(3)
                .any(|&s| s.chars().any(|c| !char::is_numeric(c)))
        {
            return None;
        }

        Some(
            version
                .trim_start_matches(|p| !char::is_numeric(p))
                .to_owned(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::http_client_pool;

    #[test]
    fn unit_sanitise_version() {
        // numbers
        assert_eq!(Api::sanitise_version("1"), None);
        assert_eq!(Api::sanitise_version("2.3"), None);
        assert_eq!(Api::sanitise_version("4.5.6"), Some("4.5.6".to_owned()));
        assert_eq!(Api::sanitise_version("7.8.9-b"), Some("7.8.9-b".to_owned()));

        // wild cards
        assert_eq!(Api::sanitise_version("*"), None);
        assert_eq!(Api::sanitise_version("1.*"), None);
        assert_eq!(Api::sanitise_version("1.*.*"), None);
        assert_eq!(Api::sanitise_version("1.2.*"), None);

        // requirements
        assert_eq!(Api::sanitise_version(">=0.0.9, <0.4"), None);
    }

    #[test]
    fn discovery() {
        let version_req = semver::VersionReq::parse("1").unwrap();
        let version = semver::Version::parse("1.0.1").unwrap();

        assert!(version_req.matches(&version))
    }

    #[actix_rt::test]
    #[ignore]
    async fn integration_dependencies() -> Result<(), String> {
        let client = http_client_pool::new()?;
        let client = Api::new(&client);

        let c = client
            .get_crate("time", &semver::Version::parse("0.2.22").unwrap())
            .await?;

        println!("{:?}", c);

        assert_eq!(c.name, "time");
        assert_eq!(c.version, semver::Version::new(0, 2, 22));

        let dependencies = c.dependency;
        assert_eq!(dependencies.len(), 8);

        // concrete
        assert!(dependencies
            .iter()
            .any(|d| d.name == "const_fn" && d.version == semver::Version::new(0, 4, 2)));
        assert!(dependencies
            .iter()
            .any(|d| d.name == "standback" && d.version == semver::Version::new(0, 2, 5)));

        // inference
        assert!(dependencies
            .iter()
            .any(|d| d.name == "libc" && d.version == semver::Version::new(0, 2, 0)));
        assert!(dependencies
            .iter()
            .any(|d| d.name == "rand" && d.version == semver::Version::new(0, 7, 0)));
        assert!(dependencies
            .iter()
            .any(|d| d.name == "stdweb" && d.version == semver::Version::new(0, 4, 0)));
        assert!(dependencies
            .iter()
            .any(|d| d.name == "serde" && d.version == semver::Version::new(1, 0, 0)));
        assert!(dependencies
            .iter()
            .any(|d| d.name == "time-macros" && d.version == semver::Version::new(0, 1, 0)));
        assert!(dependencies
            .iter()
            .any(|d| d.name == "winapi" && d.version == semver::Version::new(0, 3, 0)));

        Ok(())
    }

    #[actix_rt::test]
    #[ignore]
    async fn integration_edge_case_multiple_versions() -> Result<(), String> {
        let client = http_client_pool::new()?;
        let client = Api::new(&client);

        let c = client
            .get_crate("yaml-rust", &semver::Version::parse("0.3.5").unwrap())
            .await?;

        println!("{:?}", c);

        assert_eq!(c.name, "yaml-rust");
        assert_eq!(c.version, semver::Version::new(0, 3, 5));

        let dependencies = c.dependency;
        assert_eq!(dependencies.len(), 2);

        assert!(dependencies
            .iter()
            .any(|d| d.name == "clippy" && d.version == semver::Version::new(0, 0, 2)));
        assert!(dependencies
            .iter()
            .any(|d| d.name == "linked-hash-map" && d.version == semver::Version::new(0, 0, 9)));

        Ok(())
    }
}
