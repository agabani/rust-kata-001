mod redis_cache;

use crate::cache::redis_cache::RedisCache;
use crate::domain::Crate;

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
        Ok(None)
    }

    pub(crate) async fn save_dependencies(
        &self,
        name: &str,
        version: &semver::Version,
        crates: &[Crate],
    ) -> Result<(), String> {
        Ok(())
    }

    fn get_dependencies_key(name: &str, version: &str) -> String {
        format!("dependencies:{}:{}", name, version)
    }
}
