use crate::domain::Crate;
use semver::Version;
use sqlx::{MySqlPool, Row};

pub(crate) struct CrateDataDto {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) dependencies: i32,
    pub(crate) dependency_name: Option<String>,
    pub(crate) dependency_version: Option<String>,
}

pub(crate) struct RelationalDatabase<'a> {
    pool: &'a MySqlPool,
}

impl<'a> RelationalDatabase<'a> {
    pub(crate) fn new(pool: &'a MySqlPool) -> Self {
        RelationalDatabase { pool }
    }

    pub(crate) async fn get_one_batch(
        &self,
        name_version: &[(String, Version)],
    ) -> Result<Vec<CrateDataDto>, String> {
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

        Ok(crate_deps)
    }

    pub(crate) async fn save_one(&self, c: &Crate) -> Result<(), String> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::database_pool;
    use Version;

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

        Ok(())
    }
}
