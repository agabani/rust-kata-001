use super::domain::Crate;
use std::collections::HashMap;
use std::future::Future;

pub async fn get_dependency<DGOB, DSO, AGO>(
    db_get_one_batch: impl Fn(Vec<(String, String)>) -> DGOB + Copy,
    db_save_one: impl Fn(Crate) -> DSO + Copy,
    api_get_one: impl Fn(String, String) -> AGO + Copy,
    name: String,
    version: String,
) -> Result<Vec<Crate>, String>
where
    DGOB: Future<Output = Result<HashMap<(String, String), Option<Crate>>, String>>,
    DSO: Future<Output = Result<(), String>>,
    AGO: Future<Output = Result<Crate, String>>,
{
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
            let mut results = db_get_one_batch(name_versions).await?;
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

            for (name, version) in missing_name_versions {
                let c = api_get_one(name.to_owned(), version.to_owned()).await?;

                db_save_one(c.clone()).await?;

                results.insert((name, version), Some(c));
            }

            stack = results
                .iter()
                .map(|(_, c)| c.clone().unwrap())
                .map(|c| c.dependency)
                .flatten()
                .map(|d| (d.name.to_owned(), d.version.to_string()))
                .collect();

            for ((n, v), c) in results {
                hash.insert((n, v), c.unwrap());
            }
        }
    }

    let mut x = hash.into_iter().map(|(_, c)| c).collect::<Vec<_>>();

    x.sort_by(|a, b| (&a.name, &a.version).cmp(&(&b.name, &b.version)));

    Ok(x)
}

// https://stackoverflow.com/questions/31362206/expected-bound-lifetime-parameter-found-concrete-lifetime-e0271/31365625#31365625
