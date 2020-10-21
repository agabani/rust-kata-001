mod client;

use super::domain::Crate;
use crate::graph::domain::CrateDependency;
use semver::Version;
use std::collections::HashMap;

pub async fn get_crate(
    client: &reqwest::Client,
    name: String,
    version: String,
) -> Result<Crate, String> {
    let dto = client::dependencies(client, name.to_owned(), version.to_owned()).await?;

    let mut crate_dependencies = HashMap::new();

    if let Some(dependencies) = &dto.dependencies {
        for dependency in dependencies.iter().filter(|d| d.kind == "normal") {
            let version = sanitise_version(&dependency.req);

            if let Some(version) = version {
                crate_dependencies
                    .entry((
                        dependency.crate_id.to_owned(),
                        Version::parse(&version).unwrap(),
                    ))
                    .or_insert(CrateDependency {
                        name: dependency.crate_id.to_owned(),
                        version: Version::parse(&version).unwrap(),
                    });
            } else {
                let all_versions = client::versions(client, dependency.crate_id.to_owned())
                    .await?
                    .versions
                    .unwrap()
                    .iter()
                    .map(|f| semver::Version::parse(&f.num).unwrap())
                    .collect::<Vec<_>>();

                let version_reqs = to_requirements(&dependency.req);

                let mut filtered_versions = all_versions
                    .iter()
                    .filter(|&v| version_reqs.iter().all(|vr| vr.matches(v)))
                    .collect::<Vec<_>>();

                filtered_versions.sort();

                let best_version = filtered_versions.swap_remove(0);

                crate_dependencies
                    .entry((dependency.crate_id.to_owned(), best_version.to_owned()))
                    .or_insert(CrateDependency {
                        name: dependency.crate_id.to_owned(),
                        version: best_version.to_owned(),
                    });
            }
        }
    }

    Ok(Crate {
        name,
        version: semver::Version::parse(&version).unwrap(),
        dependency: crate_dependencies.into_iter().map(|e| e.1).collect(),
    })
}

fn to_requirements(requirements: &str) -> Vec<semver::VersionReq> {
    requirements
        .split(",")
        .into_iter()
        .map(|f| semver::VersionReq::parse(f.trim()).unwrap())
        .collect::<Vec<_>>()
}

fn sanitise_version(version: &str) -> Option<String> {
    // 0        -> 0.*.*
    // 0.0      -> 0.0.*
    // 0.0.0    -> 0.0.0
    // 0.0.0-b  -> 0.0.0-b
    // >=0.0.9, <0.4 -> none

    // check for multi requirements
    if version.split(",").into_iter().count() > 1 {
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

    return Some(
        version
            .trim_start_matches(|p| !char::is_numeric(p))
            .to_owned(),
    );
}

#[cfg(test)]
mod tests {
    use crate::factory::http_client;
    use crate::graph::api::{get_crate, sanitise_version};

    #[test]
    fn unit_sanitise_version() {
        // numbers
        assert_eq!(sanitise_version("1"), None);
        assert_eq!(sanitise_version("2.3"), None);
        assert_eq!(sanitise_version("4.5.6"), Some("4.5.6".to_owned()));
        assert_eq!(sanitise_version("7.8.9-b"), Some("7.8.9-b".to_owned()));

        // wild cards
        assert_eq!(sanitise_version("*"), None);
        assert_eq!(sanitise_version("1.*"), None);
        assert_eq!(sanitise_version("1.*.*"), None);
        assert_eq!(sanitise_version("1.2.*"), None);

        // requirements
        assert_eq!(sanitise_version(">=0.0.9, <0.4"), None);
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

        let c = get_crate(&client, "time".to_owned(), "0.2.22".to_owned()).await?;

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

        let c = get_crate(&client, "yaml-rust".to_owned(), "0.3.5".to_owned()).await?;

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
