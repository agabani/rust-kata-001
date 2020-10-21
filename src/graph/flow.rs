use super::domain::Crate;
use std::collections::HashMap;
use std::future::Future;

pub async fn get_dependency<DGO, DSO, AGO>(
    db_get_one: impl Fn(String, String) -> DGO + Copy,
    db_save_one: impl Fn(Crate) -> DSO + Copy,
    api_get_one: impl Fn(String, String) -> AGO + Copy,
    name: String,
    version: String,
) -> Result<Vec<Crate>, String>
where
    DGO: Future<Output = Result<Option<Crate>, String>>,
    DSO: Future<Output = Result<(), String>>,
    AGO: Future<Output = Result<Crate, String>>,
{
    let mut hash = HashMap::new();
    let mut stack = Vec::new();

    let parent = get_one(db_get_one, db_save_one, api_get_one, name, version).await?;

    let mut d = parent
        .dependency
        .iter()
        .map(|c| (c.name.to_owned(), c.version.to_owned()))
        .collect::<Vec<_>>();
    stack.append(&mut d);
    hash.insert(
        (parent.name.to_owned(), parent.version.to_owned()),
        parent.to_owned(),
    );

    while !stack.is_empty() {
        if let Some((name, version)) = stack.pop() {
            if !hash.contains_key(&(name.to_owned(), version.to_owned())) {
                let mut c = get_one(
                    db_get_one,
                    db_save_one,
                    api_get_one,
                    name.to_owned(),
                    version.to_owned().to_string(),
                )
                .await?;

                c.dependency
                    .sort_by(|a, b| (&a.name, &a.version).cmp(&(&b.name, &b.version)));

                let mut d = c
                    .dependency
                    .iter()
                    .map(|c| (c.name.to_owned(), c.version.to_owned()))
                    .collect::<Vec<_>>();

                stack.append(&mut d);
                hash.insert((c.name.to_owned(), c.version.to_owned()), c.to_owned());
            }
        }
    }

    let mut x = hash.into_iter().map(|(_, c)| c).collect::<Vec<_>>();

    x.sort_by(|a, b| (&a.name, &a.version).cmp(&(&b.name, &b.version)));

    Ok(x)
}

async fn get_one<DGO, DSO, AGO>(
    db_get_one: impl Fn(String, String) -> DGO,
    db_save_one: impl Fn(Crate) -> DSO,
    api_get_one: impl Fn(String, String) -> AGO,
    name: String,
    version: String,
) -> Result<Crate, String>
where
    DGO: Future<Output = Result<Option<Crate>, String>>,
    DSO: Future<Output = Result<(), String>>,
    AGO: Future<Output = Result<Crate, String>>,
{
    let fn_name = "get_one";

    let database_crate = db_get_one(name.to_owned(), version.to_owned()).await?;

    if let Some(database_crate) = database_crate {
        log::info!("{}: database_create={:?}", fn_name, database_crate);
        return Ok(database_crate);
    }

    let api_crate = api_get_one(name.to_owned(), version.to_owned()).await?;

    db_save_one(api_crate.clone()).await?;

    Ok(api_crate)
}

// https://stackoverflow.com/questions/31362206/expected-bound-lifetime-parameter-found-concrete-lifetime-e0271/31365625#31365625
