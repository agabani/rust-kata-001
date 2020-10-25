mod relational_database;

use crate::domain::{Crate, CrateDependency};
use crate::persistence::relational_database::{CrateDataDto, RelationalDatabase};
use semver::Version;
use sqlx::MySqlPool;
use std::collections::HashMap;

pub(crate) struct Persistence<'a> {
    relational_database: RelationalDatabase<'a>,
}

impl<'a> Persistence<'a> {
    pub(crate) fn new(pool: &'a MySqlPool) -> Self {
        Self {
            relational_database: RelationalDatabase::new(pool),
        }
    }

    pub(crate) async fn get_one_batch(
        &self,
        name_version: &[(String, Version)],
    ) -> Result<HashMap<(String, Version), Option<Crate>>, String> {
        let crate_deps = self.relational_database.get_one_batch(name_version).await?;

        let mut results = HashMap::new();

        for (name, version) in name_version {
            results.insert((name.to_owned(), version.to_owned()), None);
        }

        let crates = Self::transform_to_domain(&crate_deps);

        for c in crates {
            results.insert((c.name.to_owned(), c.version.to_owned()), Some(c));
        }

        Ok(results)
    }

    pub(crate) async fn save_one(&self, c: &Crate) -> Result<(), String> {
        self.relational_database.save_one(c).await
    }

    fn transform_to_domain(dtos: &[CrateDataDto]) -> Vec<Crate> {
        let fn_name = "transform_to_domain";

        // group
        let mut groups = HashMap::new();
        for dto in dtos {
            groups
                .entry((&dto.name, &dto.version))
                .or_insert_with(Vec::new)
                .push(dto);
        }

        let mut result = Vec::new();

        // transform
        for ((name, version), group) in groups {
            // if check sum fails, skips.
            if let Some(&g) = group.first() {
                if g.dependencies as usize != group.len()
                    && g.dependency_name.is_some()
                    && g.dependency_version.is_some()
                {
                    log::warn!(
                        "{}: checksum failed: name={:?} version={:?} expected={:?} actual={:?}",
                        fn_name,
                        name,
                        version,
                        g.dependencies,
                        group.len()
                    );
                    continue;
                }
            }

            let mut web_dto = Crate {
                name: name.to_string(),
                version: Version::parse(version).unwrap(),
                dependency: Vec::new(),
            };

            for item in group {
                if let Some(name) = &item.dependency_name {
                    if let Some(version) = &item.dependency_version {
                        web_dto.dependency.push(CrateDependency {
                            name: name.to_owned(),
                            version: Version::parse(version).unwrap(),
                        });
                    }
                }
            }

            result.push(web_dto);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform() {
        let input = vec![
            CrateDataDto {
                name: "name 1".to_owned(),
                version: "1.0.0".to_owned(),
                dependencies: 2,
                dependency_name: Some("sub name 1".to_owned()),
                dependency_version: Some("0.0.1".to_owned()),
            },
            CrateDataDto {
                name: "name 1".to_owned(),
                version: "1.0.0".to_owned(),
                dependencies: 2,
                dependency_name: Some("sub name 2".to_owned()),
                dependency_version: Some("0.0.2".to_owned()),
            },
            CrateDataDto {
                name: "name 2".to_owned(),
                version: "2.0.0".to_owned(),
                dependencies: 1,
                dependency_name: Some("sub name 1".to_owned()),
                dependency_version: Some("0.0.1".to_owned()),
            },
            CrateDataDto {
                name: "name 3".to_owned(),
                version: "3.0.0".to_owned(),
                dependencies: 3,
                dependency_name: Some("sub name 1".to_owned()),
                dependency_version: Some("0.0.1".to_owned()),
            },
        ];

        let expected = vec![
            Crate {
                name: "name 1".to_owned(),
                version: Version::parse("1.0.0").unwrap(),
                dependency: vec![
                    CrateDependency {
                        name: "sub name 1".to_owned(),
                        version: Version::parse("0.0.1").unwrap(),
                    },
                    CrateDependency {
                        name: "sub name 2".to_owned(),
                        version: Version::parse("0.0.2").unwrap(),
                    },
                ],
            },
            Crate {
                name: "name 2".to_owned(),
                version: Version::parse("2.0.0").unwrap(),
                dependency: vec![CrateDependency {
                    name: "sub name 1".to_owned(),
                    version: Version::parse("0.0.1").unwrap(),
                }],
            },
        ];

        let mut actual = Persistence::transform_to_domain(&input);

        actual.sort_by(|a, b| (&a.name, &a.version).cmp(&(&b.name, &b.version)));
        for d in actual.iter_mut() {
            d.dependency
                .sort_by(|a, b| (&a.name, &a.version).cmp(&(&b.name, &b.version)));
        }

        assert_eq!(actual, expected);
    }
}
