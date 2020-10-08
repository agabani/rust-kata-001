use super::domain::Crate;
use std::collections::HashMap;
use std::future::Future;

pub async fn get_dependency<F, G, H>(
    db_get_one: impl Fn(String, String) -> F + Copy,
    db_save_one: impl Fn(Crate) -> G + Copy,
    api_get_one: impl Fn(String, String) -> H + Copy,
    name: String,
    version: String,
) -> Result<Vec<Crate>, String>
where
    F: Future<Output = Result<Option<Crate>, sqlx::Error>>,
    G: Future<Output = Result<(), sqlx::Error>>,
    H: Future<Output = Result<Crate, String>>,
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
                let c = get_one(
                    db_get_one,
                    db_save_one,
                    api_get_one,
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

async fn get_one<F, G, H>(
    db_get_one: impl Fn(String, String) -> F,
    db_save_one: impl Fn(Crate) -> G,
    api_get_one: impl Fn(String, String) -> H,
    name: String,
    version: String,
) -> Result<Crate, String>
where
    F: Future<Output = Result<Option<Crate>, sqlx::Error>>,
    G: Future<Output = Result<(), sqlx::Error>>,
    H: Future<Output = Result<Crate, String>>,
{
    let database = db_get_one(name.to_owned(), version.to_owned()).await;

    match database {
        Ok(c) => {
            log::info!("{:?}", c);

            if let Some(c) = c {
                return Ok(c);
            }
        }
        Err(e) => {
            log::error!("{:?}", e);
            return Err(format!("{:?}", e));
        }
    }

    let api = api_get_one(name.to_owned(), version.to_owned()).await;

    match api {
        Ok(c) => {
            log::info!("{:?}", c);

            match db_save_one(c.clone()).await {
                Ok(_) => {}
                Err(e) => log::error!("{:?}", e),
            }

            Ok(c)
        }
        Err(e) => {
            log::error!("{:?}", e);
            Err(e)
        }
    }
}

// https://stackoverflow.com/questions/31362206/expected-bound-lifetime-parameter-found-concrete-lifetime-e0271/31365625#31365625
