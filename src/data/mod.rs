use crate::api::Api;
use crate::cache::Cache;
use crate::domain::Crate;
use crate::persistence::Persistence;
use semver::Version;
use std::collections::HashMap;

pub(crate) struct Data<'a> {
    api: Api<'a>,
    cache: Cache<'a>,
    persistence: Persistence<'a>,
}

impl<'a> Data<'a> {
    pub(crate) fn new(
        database_pool: &'a sqlx::MySqlPool,
        http_client_pool: &'a reqwest::Client,
        redis_pool: &'a redis::aio::MultiplexedConnection,
    ) -> Self {
        Self {
            api: Api::new(http_client_pool),
            cache: Cache::new(redis_pool),
            persistence: Persistence::new(database_pool),
        }
    }

    pub(crate) async fn get_dependency_graph(
        &self,
        name: String,
        version: Version,
    ) -> Result<Vec<Crate>, String> {
        let fn_name = "get_dependency_graph";

        if let Some(results) = self.cache.get_dependencies(&name, &version).await? {
            return Ok(results);
        }

        let mut hash: HashMap<(String, Version), Crate> = HashMap::new();
        let mut stack: Vec<(String, Version)> = Vec::new();
        stack.push((name.to_owned(), version.to_owned()));

        while !&stack.is_empty() {
            let name_versions = stack
                .iter()
                .filter(|&name_version| !hash.contains_key(name_version))
                .map(|(name, version)| (name.to_owned(), version.to_owned()))
                .collect::<Vec<_>>();

            stack.clear();

            if !name_versions.is_empty() {
                let mut results = self.persistence.get_one_batch(&name_versions).await?;
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

                let api_crates = futures::future::join_all(missing_name_versions.iter().map(
                    |(name, version)| {
                        let name = name.to_owned();
                        let version = version.to_owned();
                        async move {
                            let c = self.api.get_crate(&name, &version).await?;

                            self.persistence.save_one(&c).await?;

                            Ok::<Crate, String>(c)
                        }
                    },
                ))
                .await;

                for api_crate_result in api_crates {
                    if let Err(e) = api_crate_result {
                        return Err(e);
                    }

                    let api_crate = api_crate_result.unwrap();

                    results.insert(
                        (api_crate.name.to_owned(), api_crate.version.to_owned()),
                        Some(api_crate.clone()),
                    );
                }

                stack = results
                    .iter()
                    .map(|(_, c)| c.clone().unwrap())
                    .map(|c| c.dependency)
                    .flatten()
                    .map(|d| (d.name, d.version))
                    .collect();

                for ((n, v), c) in results {
                    let mut c = c.unwrap();
                    c.dependency
                        .sort_by(|a, b| (&a.name, &a.version).cmp(&(&b.name, &b.version)));
                    hash.insert((n, v), c);
                }
            }
        }

        let mut results = hash.into_iter().map(|(_, c)| c).collect::<Vec<_>>();

        results.sort_by(|a, b| (&a.name, &a.version).cmp(&(&b.name, &b.version)));

        self.cache
            .save_dependencies(&name, &version, &results)
            .await?;

        Ok(results)
    }
}
