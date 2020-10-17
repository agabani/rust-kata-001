mod client;

use super::domain::Crate;
use crate::graph::domain::CrateDependency;
use semver::Version;
use std::collections::HashMap;

pub async fn dependencies(name: String, version: String) -> Result<Crate, String> {
    let dto = client::dependencies(name.to_owned(), version.to_owned()).await?;

    let mut crate_dependencies = HashMap::new();

    if let Some(dependencies) = dto.dependencies {
        for dependency in dependencies.iter().filter(|d| d.kind == "normal") {
            let version = sanitise_version(&dependency.req);

            let version_components = version.split('.').collect::<Vec<_>>();

            if version_components.len() >= 3
                && version_components.iter().take(3).all(|&p| !p.contains('*'))
            {
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

pub async fn versions(name: String) -> Result<Vec<semver::Version>, String> {
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
