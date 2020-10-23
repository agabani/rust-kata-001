use super::domain::Crate;
use std::collections::HashMap;

#[async_trait::async_trait]
pub trait ApiGetOne {
    async fn execute(&self, name: String, version: String) -> Result<Crate, String>;
}

#[async_trait::async_trait]
pub trait DatabaseGetOneBatch {
    async fn execute(
        &self,
        crates: Vec<(String, String)>,
    ) -> Result<HashMap<(String, String), Option<Crate>>, String>;
}

#[async_trait::async_trait]
pub trait DatabaseSaveOne {
    async fn execute(&self, c: Crate) -> Result<(), String>;
}

pub async fn get_dependency(
    db_get_one_batch: &impl DatabaseGetOneBatch,
    db_save_one: &impl DatabaseSaveOne,
    api_get_one: &impl ApiGetOne,
    name: String,
    version: String,
) -> Result<Vec<Crate>, String> {
    let fn_name = "get_dependency";

    let mut hash: HashMap<(String, String), Crate> = HashMap::new();
    let mut stack: Vec<(String, String)> = Vec::new();
    stack.push((name, version));

    while !&stack.is_empty() {
        let name_versions = stack
            .iter()
            .filter(|&name_version| !hash.contains_key(name_version))
            .map(|(name, version)| (name.to_owned(), version.to_owned()))
            .collect::<Vec<_>>();

        stack.clear();

        if !name_versions.is_empty() {
            let mut results = db_get_one_batch.execute(name_versions).await?;
            log::info!("{}: database_create={:?}", fn_name, results);

            let missing_name_versions = results
                .iter()
                .filter_map(|((n, v), c)| {
                    if c.is_none() {
                        Some((n.to_owned(), v.to_owned()))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            let futures = missing_name_versions
                .iter()
                .map(|(name, version)| {
                    let name = name.to_owned();
                    let version = version.to_owned();
                    async move {
                        let c = api_get_one.execute(name, version).await?;

                        db_save_one.execute(c.clone()).await?;

                        Ok::<Crate, String>(c)
                    }
                })
                .collect::<Vec<_>>();

            let api_crates = futures::future::join_all(futures).await;

            for api_crate_result in api_crates {
                if let Err(e) = api_crate_result {
                    return Err(e);
                }

                let api_crate = api_crate_result.unwrap();

                results.insert(
                    (api_crate.name.to_owned(), api_crate.version.to_string()),
                    Some(api_crate.clone()),
                );
            }

            stack = results
                .iter()
                .map(|(_, c)| c.clone().unwrap())
                .map(|c| c.dependency)
                .flatten()
                .map(|d| (d.name.to_owned(), d.version.to_string()))
                .collect();

            for ((n, v), c) in results {
                let mut c = c.unwrap();
                c.dependency
                    .sort_by(|a, b| (&a.name, &a.version).cmp(&(&b.name, &b.version)));
                hash.insert((n, v), c);
            }
        }
    }

    let mut x = hash.into_iter().map(|(_, c)| c).collect::<Vec<_>>();

    x.sort_by(|a, b| (&a.name, &a.version).cmp(&(&b.name, &b.version)));

    Ok(x)
}

// https://stackoverflow.com/questions/31362206/expected-bound-lifetime-parameter-found-concrete-lifetime-e0271/31365625#31365625
