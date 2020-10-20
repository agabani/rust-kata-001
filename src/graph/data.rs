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
    ) -> Result<Option<Crate>, String> {
        let fn_name = "get_one";

        log::info!("{}: name={} version={}", fn_name, name, version);

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
        .await
        .map_err(|e| {
            log::error!("{}: error {:?}", fn_name, e);
            format!("{}: error {:?}", fn_name, e)
        })?;

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

    pub async fn save_one(pool: &MySqlPool, c: Crate) -> Result<(), String> {
        let fn_name = "save_one";

        log::info!("{}: crate={:?}", fn_name, c);

        let transaction = pool.begin().await.map_err(|e| {
            log::error!("{}: error {:?}", fn_name, e);
            format!("{}: error {:?}", fn_name, e)
        })?;

        sqlx::query("INSERT INTO crate (name, version) VALUE (?, ?)")
            .bind(c.name)
            .bind(c.version.to_string())
            .execute(pool)
            .await
            .map_err(|e| {
                log::error!("{}: error {:?}", fn_name, e);
                format!("{}: error {:?}", fn_name, e)
            })?;

        let row = sqlx::query("SELECT LAST_INSERT_ID()")
            .fetch_one(pool)
            .await
            .map_err(|e| {
                log::error!("{}: error {:?}", fn_name, e);
                format!("{}: error {:?}", fn_name, e)
            })?;

        let id: u64 = row.get(0);

        for d in c.dependency {
            sqlx::query("INSERT INTO crate_dependency (crate_id, name, version) VALUE (?, ?, ?)")
                .bind(id)
                .bind(d.name)
                .bind(d.version.to_string())
                .execute(pool)
                .await
                .map_err(|e| {
                    log::error!("{}: error {:?}", fn_name, e);
                    format!("{}: error {:?}", fn_name, e)
                })?;
        }

        transaction.commit().await.map_err(|e| {
            log::error!("{}: error {:?}", fn_name, e);
            format!("{}: error {:?}", fn_name, e)
        })?;

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
                version: "1.0.0".to_owned(),
                dependency_name: Some("sub name 1".to_owned()),
                dependency_version: Some("0.0.1".to_owned()),
            },
            CrateDataDto {
                name: "name 1".to_owned(),
                version: "1.0.0".to_owned(),
                dependency_name: Some("sub name 2".to_owned()),
                dependency_version: Some("0.0.2".to_owned()),
            },
            CrateDataDto {
                name: "name 2".to_owned(),
                version: "2.0.0".to_owned(),
                dependency_name: Some("sub name 1".to_owned()),
                dependency_version: Some("0.0.1".to_owned()),
            },
        ];

        let expected = vec![
            Crate {
                name: "name 1".to_owned(),
                version: semver::Version::parse("1.0.0").unwrap(),
                dependency: vec![
                    CrateDependency {
                        name: "sub name 1".to_owned(),
                        version: semver::Version::parse("0.0.1").unwrap(),
                    },
                    CrateDependency {
                        name: "sub name 2".to_owned(),
                        version: semver::Version::parse("0.0.2").unwrap(),
                    },
                ],
            },
            Crate {
                name: "name 2".to_owned(),
                version: semver::Version::parse("2.0.0").unwrap(),
                dependency: vec![CrateDependency {
                    name: "sub name 1".to_owned(),
                    version: semver::Version::parse("0.0.1").unwrap(),
                }],
            },
        ];

        let mut actual = CrateDataDto::transform_to_domain(&input);

        actual.sort_by(|a, b| (&a.name, &a.version).cmp(&(&b.name, &b.version)));
        for d in actual.iter_mut() {
            d.dependency
                .sort_by(|a, b| (&a.name, &a.version).cmp(&(&b.name, &b.version)));
        }

        assert_eq!(actual, expected);
    }
}
