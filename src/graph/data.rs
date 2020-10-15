use super::domain::{Crate, CrateDependency};
use sqlx::{MySqlPool, Row};
use std::collections::HashMap;

pub struct CrateDataDto {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) dependency_name: Option<String>,
    pub(crate) dependency_version: Option<String>,
}

impl CrateDataDto {
    pub async fn get_one(
        pool: &MySqlPool,
        name: String,
        version: String,
    ) -> Result<Option<Crate>, sqlx::Error> {
        let records = sqlx::query(
            "SELECT c.name, c.version, cd.name, cd.version
FROM crate AS c
         LEFT JOIN crate_dependency cd on c.id = cd.crate_id
WHERE c.name = ?
  AND c.version = ?",
        )
        .bind(name)
        .bind(version)
        .fetch_all(pool)
        .await?;

        let mut crate_deps = Vec::new();

        for record in records {
            crate_deps.push(CrateDataDto {
                name: record.get(0),
                version: record.get(1),
                dependency_name: record.get(2),
                dependency_version: record.get(3),
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
        let transaction = pool.begin().await?;

        sqlx::query("INSERT INTO crate (name, version) VALUE (?, ?)")
            .bind(c.name)
            .bind(c.version.to_string())
            .execute(pool)
            .await?;

        let row = sqlx::query("SELECT LAST_INSERT_ID()")
            .fetch_one(pool)
            .await?;

        let id: u64 = row.get(0);

        for d in c.dependency {
            sqlx::query("INSERT INTO crate_dependency (crate_id, name, version) VALUE (?, ?, ?)")
                .bind(id)
                .bind(d.name)
                .bind(d.version.to_string())
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
                if let Some(name) = &item.dependency_name {
                    if let Some(version) = &item.dependency_version {
                        web_dto.dependency.push(CrateDependency {
                            name: name.to_owned(),
                            version: semver::Version::parse(version).unwrap(),
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
                version: "version 1".to_owned(),
                dependency_name: Some("sub name 1".to_owned()),
                dependency_version: Some("sub version 1".to_owned()),
            },
            CrateDataDto {
                name: "name 1".to_owned(),
                version: "version 1".to_owned(),
                dependency_name: Some("sub name 2".to_owned()),
                dependency_version: Some("sub version 2".to_owned()),
            },
            CrateDataDto {
                name: "name 2".to_owned(),
                version: "version 2".to_owned(),
                dependency_name: Some("sub name 1".to_owned()),
                dependency_version: Some("sub version 1".to_owned()),
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
