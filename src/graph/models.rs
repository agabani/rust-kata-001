use crate::graph::domain::Crate;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ErrorWebDto {
    pub status_code: i32,
    pub error_message: String,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct CrateWebDto {
    name: String,
    version: String,
    dependency: Vec<CrateDependencyWebDto>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct CrateDependencyWebDto {
    name: String,
    version: String,
}

#[derive(Deserialize)]
pub struct ListQueryParams {
    pub name: Option<String>,
    pub version: Option<String>,
}

impl CrateWebDto {
    pub fn transform(c: &Crate) -> Self {
        Self {
            name: c.name.clone(),
            version: c.version.to_string(),
            dependency: c
                .dependency
                .iter()
                .map(|d| CrateDependencyWebDto {
                    name: d.name.clone(),
                    version: d.version.to_string(),
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::domain::{Crate, CrateDependency};

    #[test]
    fn transform() {
        let input = Crate {
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

        let expected = CrateWebDto {
            name: "name".to_owned(),
            version: "version".to_owned(),
            dependency: vec![
                CrateDependencyWebDto {
                    name: "sub name 1".to_owned(),
                    version: "sub version 1".to_owned(),
                },
                CrateDependencyWebDto {
                    name: "sub name 2".to_owned(),
                    version: "sub version 2".to_owned(),
                },
            ],
        };

        let actual = CrateWebDto::transform(&input);

        assert_eq!(actual, expected)
    }
}
