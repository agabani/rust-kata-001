mod client;

use super::domain::Crate;
use crate::graph::api::client::DependencyApiDto;
use crate::graph::domain::CrateDependency;
pub use client::CratesIoClient;
use semver::Version;
use std::collections::HashMap;

pub struct Client<'a> {
    crates_io_client: CratesIoClient<'a>,
}

impl<'a> Client<'a> {
    pub fn new(client: &'a reqwest::Client) -> Client<'a> {
        Client {
            crates_io_client: CratesIoClient::new(client),
        }
    }

    pub async fn get_crate(&self, name: &str, version: &str) -> Result<Crate, String> {
        let dto = self.crates_io_client.dependencies(name, version).await?;

        let mut crate_dependencies = HashMap::new();

        if let Some(dependencies) = dto.dependencies {
            let results = futures::future::join_all(
                dependencies
                    .iter()
                    .filter(|&d| d.kind == "normal")
                    .map(|dependency| self.from_dto_to_domain(dependency)),
            )
            .await
            .iter_mut()
            .map(|result| result.clone().unwrap())
            .collect::<Vec<_>>();

            for crate_dependency in results.iter() {
                crate_dependencies
                    .entry((
                        crate_dependency.name.to_owned(),
                        crate_dependency.version.to_owned(),
                    ))
                    .or_insert(CrateDependency {
                        name: crate_dependency.name.to_owned(),
                        version: crate_dependency.version.to_owned(),
                    });
            }
        }

        Ok(Crate {
            name: name.to_owned(),
            version: semver::Version::parse(&version).unwrap(),
            dependency: crate_dependencies.into_iter().map(|e| e.1).collect(),
        })
    }

    async fn from_dto_to_domain(
        &self,
        dependency: &DependencyApiDto,
    ) -> Result<CrateDependency, String> {
        let version = Self::sanitise_version(&dependency.req);

        if let Some(version) = version {
            return Ok(CrateDependency {
                name: dependency.crate_id.to_owned(),
                version: Version::parse(&version).unwrap(),
            });
        }

        let all_versions = self
            .crates_io_client
            .versions(&dependency.crate_id)
            .await?
            .versions
            .unwrap()
            .iter()
            .map(|f| semver::Version::parse(&f.num).unwrap())
            .collect::<Vec<_>>();

        let version_reqs = self.to_requirements(&dependency.req);

        let mut filtered_versions = all_versions
            .iter()
            .filter(|&v| version_reqs.iter().all(|vr| vr.matches(v)))
            .collect::<Vec<_>>();

        filtered_versions.sort();

        let best_version = filtered_versions.swap_remove(0);

        Ok(CrateDependency {
            name: dependency.crate_id.to_owned(),
            version: best_version.to_owned(),
        })
    }

    fn to_requirements(&self, requirements: &str) -> Vec<semver::VersionReq> {
        requirements
            .split(',')
            .into_iter()
            .map(|f| semver::VersionReq::parse(f.trim()).unwrap())
            .collect::<Vec<_>>()
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
    use crate::factory::http_client;
    use crate::graph::api::Client;

    #[test]
    fn unit_sanitise_version() {
        // numbers
        assert_eq!(Client::sanitise_version("1"), None);
        assert_eq!(Client::sanitise_version("2.3"), None);
        assert_eq!(Client::sanitise_version("4.5.6"), Some("4.5.6".to_owned()));
        assert_eq!(
            Client::sanitise_version("7.8.9-b"),
            Some("7.8.9-b".to_owned())
        );

        // wild cards
        assert_eq!(Client::sanitise_version("*"), None);
        assert_eq!(Client::sanitise_version("1.*"), None);
        assert_eq!(Client::sanitise_version("1.*.*"), None);
        assert_eq!(Client::sanitise_version("1.2.*"), None);

        // requirements
        assert_eq!(Client::sanitise_version(">=0.0.9, <0.4"), None);
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
        let client = http_client::new()?;
        let client = Client::new(&client);

        let c = client.get_crate("time", "0.2.22").await?;

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
        let client = http_client::new()?;
        let client = Client::new(&client);

        let c = client.get_crate("yaml-rust", "0.3.5").await?;

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
