use super::domain::{Crate, CrateDependency};
use sqlx::{MySqlPool, Row};
use std::collections::HashMap;

pub struct CrateDataDto {
    pub(crate) id: i32,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) dependency_name: String,
    pub(crate) dependency_version: String,
}

impl CrateDataDto {
    pub async fn get_one(
        pool: &MySqlPool,
        name: String,
        version: String,
    ) -> Result<Option<Crate>, sqlx::Error> {
        let records = sqlx::query("SELECT * FROM crate_deps WHERE name = ? AND version = ?")
            .bind(name)
            .bind(version)
            .fetch_all(pool)
            .await?;

        let mut crate_deps = Vec::new();

        for record in records {
            crate_deps.push(CrateDataDto {
                id: record.get(0),
                name: record.get(1),
                version: record.get(2),
                dependency_name: record.get(3),
                dependency_version: record.get(4),
            });
        }

        let mut crates = Self::transform_to_domain(&crate_deps);

        if crates.get(0).is_none() {
            Ok(None)
        } else {
            Ok(Some(crates.swap_remove(0)))
        }
    }

    pub async fn save_one(pool: &MySqlPool, c: Crate) -> Result<(), sqlx::Error> {
        let dtos = Self::transform_to_data(&[&c]);

        let transaction = pool.begin().await?;

        for dto in dtos {
            sqlx::query("INSERT INTO crate_deps (name, version, dependency_name, dependency_version) VALUE (?, ?, ?, ?)")
                .bind(dto.name)
                .bind(dto.version)
                .bind(dto.dependency_name)
                .bind(dto.dependency_version)
                .execute(pool)
                .await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    fn transform_to_domain(dtos: &[Self]) -> Vec<Crate> {
        let mut result = Vec::new();

        // group
        let mut groups = HashMap::new();
        for dto in dtos {
            groups
                .entry((&dto.name, &dto.version))
                .or_insert_with(Vec::new)
                .push(dto);
        }

        // transform
        for ((name, version), group) in groups {
            let mut web_dto = Crate {
                name: name.to_string(),
                version: semver::Version::parse(version).unwrap(),
                dependency: Vec::new(),
            };

            for item in group {
                web_dto.dependency.push(CrateDependency {
                    name: item.dependency_name.clone(),
                    version: semver::Version::parse(&item.dependency_version).unwrap(),
                });
            }

            result.push(web_dto);
        }

        result
    }

    fn transform_to_data(crates: &[&Crate]) -> Vec<Self> {
        let mut dtos = Vec::new();

        for c in crates {
            for d in &c.dependency {
                dtos.push(CrateDataDto {
                    id: 0,
                    name: c.name.clone(),
                    version: c.version.to_string(),
                    dependency_name: d.name.clone(),
                    dependency_version: d.version.to_string(),
                })
            }
        }

        dtos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform() {
        let input = vec![
            CrateDataDto {
                id: 1,
                name: "name 1".to_owned(),
                version: "version 1".to_owned(),
                dependency_name: "sub name 1".to_owned(),
                dependency_version: "sub version 1".to_owned(),
            },
            CrateDataDto {
                id: 2,
                name: "name 1".to_owned(),
                version: "version 1".to_owned(),
                dependency_name: "sub name 2".to_owned(),
                dependency_version: "sub version 2".to_owned(),
            },
            CrateDataDto {
                id: 2,
                name: "name 2".to_owned(),
                version: "version 2".to_owned(),
                dependency_name: "sub name 1".to_owned(),
                dependency_version: "sub version 1".to_owned(),
            },
        ];

        let expected = vec![
            Crate {
                name: "name 1".to_owned(),
                version: semver::Version::parse("version 1").unwrap(),
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
            },
            Crate {
                name: "name 2".to_owned(),
                version: semver::Version::parse("version 2").unwrap(),
                dependency: vec![CrateDependency {
                    name: "sub name 1".to_owned(),
                    version: semver::Version::parse("sub version 1").unwrap(),
                }],
            },
        ];

        let actual = CrateDataDto::transform_to_domain(&input);

        assert_eq!(actual, expected);
    }
}
