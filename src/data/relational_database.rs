use crate::domain::{Crate, CrateDependency};
use semver::Version;
use sqlx::{MySqlPool, Row};
use std::collections::HashMap;

struct CrateDataDto {
    name: String,
    version: String,
    dependencies: i32,
    dependency_name: Option<String>,
    dependency_version: Option<String>,
}

pub struct RelationalDatabase<'a> {
    pool: &'a MySqlPool,
}

impl<'a> RelationalDatabase<'a> {
    pub fn new(pool: &'a MySqlPool) -> Self {
        RelationalDatabase { pool }
    }

    pub async fn get_one_batch(
        &self,
        name_version: &[(String, Version)],
    ) -> Result<HashMap<(String, Version), Option<Crate>>, String> {
        let fn_name = "get_many";

        let mut sql = "SELECT c.name, c.version, c.dependencies, cd.name, cd.version
FROM crate c
         LEFT JOIN crate_dependency cd on c.id = cd.crate_id
WHERE (c.name = ? AND c.version = ?)"
            .to_string();

        for _ in 1..name_version.len() {
            sql += "
   OR (c.name = ? AND c.version = ?)";
        }

        let mut query = sqlx::query(&sql);

        for (name, version) in name_version {
            query = query.bind(name).bind(version.to_string());
        }

        let records = query.fetch_all(self.pool).await.map_err(|e| {
            log::error!("{}: error {:?}", fn_name, e);
            format!("{}: error {:?}", fn_name, e)
        })?;

        let mut crate_deps = Vec::new();

        for record in records {
            crate_deps.push(CrateDataDto {
                name: record.get(0),
                version: record.get(1),
                dependencies: record.get(2),
                dependency_name: record.get(3),
                dependency_version: record.get(4),
            });
        }

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

    pub async fn save_one(&self, c: &Crate) -> Result<(), String> {
        let fn_name = "save_one";

        log::info!("{}: crate={:?}", fn_name, c);

        sqlx::query(
            "INSERT INTO crate (name, version, dependencies) VALUE (?, ?, ?)
ON DUPLICATE KEY UPDATE id=LAST_INSERT_ID(id)",
        )
        .bind(&c.name)
        .bind(c.version.to_string())
        .bind(c.dependency.len() as i32)
        .execute(self.pool)
        .await
        .map_err(|e| {
            log::error!("{}: error {:?}", fn_name, e);
            format!("{}: error {:?}", fn_name, e)
        })?;

        let row = sqlx::query(
            "SELECT c.id
FROM crate c
WHERE name = ?
  AND version = ?",
        )
        .bind(&c.name)
        .bind(c.version.to_string())
        .fetch_one(self.pool)
        .await
        .map_err(|e| {
            log::error!("{}: error {:?}", fn_name, e);
            format!("{}: error {:?}", fn_name, e)
        })?;

        let id: i32 = row.get(0);

        for d in &c.dependency {
            sqlx::query(
                "INSERT INTO crate_dependency (crate_id, name, version) VALUE (?, ?, ?)
ON DUPLICATE KEY UPDATE id=LAST_INSERT_ID(id)",
            )
            .bind(id)
            .bind(&d.name)
            .bind(d.version.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| {
                log::error!("{}: error {:?}", fn_name, e);
                format!("{}: error {:?}", fn_name, e)
            })?;
        }

        Ok(())
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
    use crate::factory::database_pool;
    use Version;

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

        let mut actual = RelationalDatabase::transform_to_domain(&input);

        actual.sort_by(|a, b| (&a.name, &a.version).cmp(&(&b.name, &b.version)));
        for d in actual.iter_mut() {
            d.dependency
                .sort_by(|a, b| (&a.name, &a.version).cmp(&(&b.name, &b.version)));
        }

        assert_eq!(actual, expected);
    }

    #[actix_rt::test]
    #[ignore]
    async fn integration_get_one_batch() -> Result<(), String> {
        let pool = database_pool::new("mysql://root:password@localhost:3306/rust-kata-001").await?;
        let database = RelationalDatabase::new(&pool);

        let crates = database
            .get_one_batch(&vec![
                ("actix-web".to_owned(), Version::new(3, 1, 0)),
                ("rand".to_owned(), Version::new(0, 7, 3)),
                ("syn".to_owned(), Version::new(1, 0, 33)),
            ])
            .await?;

        assert_eq!(crates.len(), 3, "expected 3 crates");

        let actix_web = crates
            .get(&("actix-web".to_owned(), Version::new(3, 1, 0)))
            .clone()
            .expect("entry was not in the response")
            .clone()
            .expect("entry was not in the database");
        assert_eq!(actix_web.name, "actix-web");
        assert_eq!(actix_web.version, Version::new(3, 1, 0));
        assert!(!actix_web.dependency.is_empty());

        let rand = crates
            .get(&("rand".to_owned(), Version::new(0, 7, 3)))
            .clone()
            .expect("entry was not in the response")
            .clone()
            .expect("entry was not in the database");
        assert_eq!(rand.name, "rand");
        assert_eq!(rand.version, Version::new(0, 7, 3));
        assert!(!rand.dependency.is_empty());

        let syn = crates
            .get(&("syn".to_owned(), Version::new(1, 0, 33)))
            .clone()
            .expect("entry was not in the response")
            .clone()
            .expect("entry was not in the database");
        assert_eq!(syn.name, "syn");
        assert_eq!(syn.version, Version::new(1, 0, 33));
        assert!(!syn.dependency.is_empty());
        assert!(
            syn.dependency
                .iter()
                .any(|d| d.name == "proc-macro2" && d.version == Version::new(1, 0, 13)),
            "dependency missing proc-macro2 v1.0.13"
        );
        assert!(
            syn.dependency
                .iter()
                .any(|d| d.name == "quote" && d.version == Version::new(1, 0, 0)),
            "dependency missing quote v1.0.0"
        );
        assert!(
            syn.dependency
                .iter()
                .any(|d| d.name == "unicode-xid" && d.version == Version::new(0, 2, 0)),
            "dependency missing unicode-xid v0.2.0"
        );

        Ok(())
    }
}
