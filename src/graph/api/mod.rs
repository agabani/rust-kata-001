mod client;

use super::domain::Crate;
use crate::graph::domain::CrateDependency;
use semver::Version;
use std::collections::HashMap;

pub async fn dependencies(
    client: &reqwest::Client,
    name: String,
    version: String,
) -> Result<Crate, String> {
    let dto = client::dependencies(client, name.to_owned(), version.to_owned()).await?;

    let mut crate_dependencies = HashMap::new();

    if let Some(dependencies) = &dto.dependencies {
        for dependency in dependencies.iter().filter(|d| d.kind == "normal") {
            let version = sanitise_version(&dependency.req);

            let version_components = version
                .split(|c: char| c == '.' || c == '-')
                .collect::<Vec<_>>();

            if version_components.iter().take(3).all(|&p| !p.contains('*')) {
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

                let version_req = semver::VersionReq::parse(&dependency.req).unwrap();

                let mut filtered_versions = all_versions
                    .iter()
                    .filter(|&v| version_req.matches(v))
                    .collect::<Vec<_>>();

                filtered_versions.sort();

                let best_version = filtered_versions.swap_remove(0);

                crate_dependencies
                    .entry((dependency.crate_id.to_owned(), best_version.to_owned()))
                    .or_insert(CrateDependency {
                        name: dependency.crate_id.to_owned(),
                        version: best_version.to_owned(),
                    });

                // TODO: do version discovery
            }
        }
    }

    Ok(Crate {
        name,
        version: semver::Version::parse(&version).unwrap(),
        dependency: crate_dependencies.into_iter().map(|e| e.1).collect(),
    })
}

pub async fn versions(_: String) -> Result<Vec<semver::Version>, String> {
    // TODO: delete this method, this package acts like an anti corruption layer to the rest of the application

    Err("Not Implemented".to_owned())
}

fn sanitise_version(version: &str) -> String {
    // 0        -> 0.*.*
    // 0.0      -> 0.0.*
    // 0.0.0    -> 0.0.0
    // 0.0.0-b  -> 0.0.0-b

    let mut dots = 2;
    let mut chars = Vec::new();

    for char in version.trim_start_matches(|p| !char::is_numeric(p)).chars() {
        if char == '.' {
            dots -= 1;
        }
        chars.push(char)
    }

    for _ in 0..dots {
        chars.push('.');
        chars.push('*');
    }

    chars.iter().collect()
}

#[cfg(test)]
mod tests {
    use crate::factory::http_client;
    use crate::graph::api::{dependencies, sanitise_version};

    #[test]
    fn unit_sanitise_version() {
        assert_eq!(sanitise_version("1"), "1.*.*");
        assert_eq!(sanitise_version("2.3"), "2.3.*");
        assert_eq!(sanitise_version("4.5.6"), "4.5.6");
        assert_eq!(sanitise_version("7.8.9-b"), "7.8.9-b");
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

        let c = dependencies(&client, "time".to_owned(), "0.2.22".to_owned()).await?;

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

        let c = dependencies(&client, "yaml-rust".to_owned(), "0.3.5".to_owned()).await?;

        println!("{:?}", c);

        assert_eq!(c.name, "yaml-rust");
        assert_eq!(c.version, semver::Version::new(0, 3, 5));

        let dependencies = c.dependency;
        assert_eq!(dependencies.len(), 8);

        Ok(())
    }

}
