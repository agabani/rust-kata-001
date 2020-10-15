use super::domain::Crate;
use semver::Version;
use std::collections::HashMap;
use std::future::Future;

pub async fn get_dependency<DGO, DSO, AGO, AGV>(
    db_get_one: impl Fn(String, String) -> DGO + Copy,
    db_save_one: impl Fn(Crate) -> DSO + Copy,
    api_get_one: impl Fn(String, String) -> AGO + Copy,
    api_get_versions: impl Fn(String) -> AGV + Copy,
    name: String,
    version: String,
) -> Result<Vec<Crate>, String>
where
    DGO: Future<Output = Result<Option<Crate>, String>>,
    DSO: Future<Output = Result<(), String>>,
    AGO: Future<Output = Result<Crate, String>>,
    AGV: Future<Output = Result<Vec<semver::Version>, String>>,
{
    let mut hash = HashMap::new();
    let mut stack = Vec::new();

    let parent = get_one(
        db_get_one,
        db_save_one,
        api_get_one,
        api_get_versions,
        name,
        version,
    )
    .await?;

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
                let c = get_one(
                    db_get_one,
                    db_save_one,
                    api_get_one,
                    api_get_versions,
                    name.to_owned(),
                    version.to_owned().to_string(),
                )
                .await?;

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

    let x = hash.into_iter().map(|(_, c)| c).collect();

    Ok(x)
}

async fn get_one<DGO, DSO, AGO, AGV>(
    db_get_one: impl Fn(String, String) -> DGO,
    db_save_one: impl Fn(Crate) -> DSO,
    api_get_one: impl Fn(String, String) -> AGO,
    api_get_versions: impl Fn(String) -> AGV + Copy,
    name: String,
    version: String,
) -> Result<Crate, String>
where
    DGO: Future<Output = Result<Option<Crate>, String>>,
    DSO: Future<Output = Result<(), String>>,
    AGO: Future<Output = Result<Crate, String>>,
    AGV: Future<Output = Result<Vec<semver::Version>, String>>,
{
    let fn_name = "get_one";

    let database_crate = db_get_one(name.to_owned(), version.to_owned()).await?;

    if let Some(database_crate) = database_crate {
        log::info!("{}: database_create={:?}", fn_name, database_crate);
        return Ok(database_crate);
    }

    let mut api_crate = api_get_one(name.to_owned(), version.to_owned()).await?;

    log::info!("{}: checking for version 0.0.0", fn_name);
    for offending_crate_dependency in api_crate
        .dependency
        .iter_mut()
        .filter(|d| d.version == Version::new(0, 0, 0))
    {
        log::info!(
            "{} offending_crate_dependency={:?}",
            fn_name,
            offending_crate_dependency
        );
        let versions = api_get_versions(offending_crate_dependency.name.to_owned())
            .await
            .unwrap();
        let min_version = versions.iter().min().unwrap();
        log::info!(
            "{} versions={:?} min_version={:?}",
            fn_name,
            versions,
            min_version
        );

        offending_crate_dependency.version = min_version.to_owned()
    }

    db_save_one(api_crate.clone()).await?;

    Ok(api_crate)
}

// https://stackoverflow.com/questions/31362206/expected-bound-lifetime-parameter-found-concrete-lifetime-e0271/31365625#31365625
