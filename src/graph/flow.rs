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
    DGO: Future<Output = Result<Option<Crate>, sqlx::Error>>,
    DSO: Future<Output = Result<(), sqlx::Error>>,
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
    DGO: Future<Output = Result<Option<Crate>, sqlx::Error>>,
    DSO: Future<Output = Result<(), sqlx::Error>>,
    AGO: Future<Output = Result<Crate, String>>,
    AGV: Future<Output = Result<Vec<semver::Version>, String>>,
{
    let database = db_get_one(name.to_owned(), version.to_owned()).await;

    match database {
        Ok(c) => {
            log::info!("get one: success from database: {:?}", c);

            if let Some(c) = c {
                return Ok(c);
            }
        }
        Err(e) => {
            log::error!("get one: failed to query database {:?}", e);
            return Err(format!("{:?}", e));
        }
    }

    let api = api_get_one(name.to_owned(), version.to_owned()).await;

    match api {
        Ok(c) => {
            log::info!("flow: get one api: {:?}", c);

            for x in c
                .dependency
                .iter()
                .filter(|&d| d.version == Version::new(0, 0, 0))
            {
                let versions = api_get_versions(x.name.to_owned()).await.unwrap();
                let min_version = versions.iter().min().unwrap();
                log::error!("........: {:?} {:?} {:?}", x, version, min_version)
            }

            match db_save_one(c.clone()).await {
                Ok(_) => {}
                Err(e) => log::error!("flow: get one: failed to save to database {:?}", e),
            }

            Ok(c)
        }
        Err(e) => {
            log::error!("flow: get one: failed to make api call {:?}", e);
            Err(e)
        }
    }
}

// https://stackoverflow.com/questions/31362206/expected-bound-lifetime-parameter-found-concrete-lifetime-e0271/31365625#31365625
