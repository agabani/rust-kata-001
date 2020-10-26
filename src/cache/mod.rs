mod redis_cache;

use crate::cache::redis_cache::RedisCache;
use crate::domain::{Crate, CrateDependency};

pub(crate) struct Cache<'a> {
    redis: RedisCache<'a>,
}

impl<'a> Cache<'a> {
    pub(crate) fn new(redis_pool: &'a redis::aio::MultiplexedConnection) -> Self {
        Cache {
            redis: RedisCache::new(redis_pool),
        }
    }

    pub(crate) async fn get_dependencies(
        &self,
        name: &str,
        version: &semver::Version,
    ) -> Result<Option<Vec<Crate>>, String> {
        let fn_name = "get_dependencies";

        if let Some(json) = self
            .redis
            .get_string(&Self::get_dependencies_key(name, version))
            .await?
        {
            let result = serde_json::from_str::<Vec<CrateDto>>(&json).map_err(|error| {
                log::error!("{}: Error={:?}", fn_name, error);
                format!("{}: Error={:?}", fn_name, error)
            })?;

            Ok(Some(result.iter().map(CrateDto::into).collect()))
        } else {
            Ok(None)
        }
    }

    pub(crate) async fn save_dependencies(
        &self,
        name: &str,
        version: &semver::Version,
        crates: &[Crate],
    ) -> Result<(), String> {
        let fn_name = "save_dependencies";

        let dto = crates.iter().map(CrateDto::from).collect::<Vec<_>>();

        let result = serde_json::to_string(&dto).map_err(|error| {
            log::error!("{}: Error={:?}", fn_name, error);
            format!("{}: Error={:?}", fn_name, error)
        })?;

        self.redis
            .set_string(&Self::get_dependencies_key(name, version), &result)
            .await?;

        Ok(())
    }

    fn get_dependencies_key(name: &str, version: &semver::Version) -> String {
        format!("dependencies:{}:{}", name, version)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CrateDto {
    name: String,
    version: String,
    dependency: Vec<CrateDependencyDto>,
}

impl CrateDto {
    fn from(item: &Crate) -> Self {
        Self {
            name: item.name.to_owned(),
            version: item.version.to_string(),
            dependency: item
                .dependency
                .iter()
                .map(CrateDependencyDto::from)
                .collect(),
        }
    }

    fn into(item: &Self) -> Crate {
        Crate {
            name: item.name.to_owned(),
            version: semver::Version::parse(&item.version).unwrap(),
            dependency: item
                .dependency
                .iter()
                .map(CrateDependencyDto::into)
                .collect(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CrateDependencyDto {
    name: String,
    version: String,
}

impl CrateDependencyDto {
    fn from(item: &CrateDependency) -> Self {
        Self {
            name: item.name.to_owned(),
            version: item.version.to_string(),
        }
    }

    fn into(item: &Self) -> CrateDependency {
        CrateDependency {
            name: item.name.to_owned(),
            version: semver::Version::parse(&item.version).unwrap(),
        }
    }
}
