use redis::Value;
use std::str::from_utf8;

struct RedisCache<'a> {
    pool: &'a redis::aio::MultiplexedConnection,
}

impl<'a> RedisCache<'a> {
    fn new(pool: &'a redis::aio::MultiplexedConnection) -> Self {
        Self { pool }
    }

    async fn get_string(&self, key: &str) -> Result<Option<String>, String> {
        let fn_name = "get_string";

        let mut connection = self.pool.clone();

        let value: Value = redis::cmd("GET")
            .arg(&[key])
            .query_async(&mut connection)
            .await
            .map_err(|error| Self::map_error(fn_name, &error))?;

        match value {
            Value::Nil => Ok(None),
            Value::Int(_) => unimplemented!("int"),
            Value::Data(data) => Ok(Some(
                from_utf8(&data)
                    .map_err(|error| format!("{:?}", error))?
                    .to_owned(),
            )),
            Value::Bulk(_) => unimplemented!("bulk"),
            Value::Status(_) => unimplemented!("status"),
            Value::Okay => unimplemented!("okay"),
        }
    }

    async fn set_string(&self, key: &str, value: &str) -> Result<(), String> {
        let fn_name = "set_string";

        let mut connection = self.pool.clone();

        let value: Value = redis::cmd("SET")
            .arg(&[key, value])
            .query_async(&mut connection)
            .await
            .map_err(|error| Self::map_error(fn_name, &error))?;

        match value {
            Value::Nil => unimplemented!("Nil"),
            Value::Int(_) => unimplemented!("Int"),
            Value::Data(_) => unimplemented!("Data"),
            Value::Bulk(_) => unimplemented!("Bulk"),
            Value::Status(_) => unimplemented!("Status"),
            Value::Okay => Ok(()),
        }
    }

    fn map_error(fn_name: &str, error: &redis::RedisError) -> String {
        log::error!("{}: RedisError={:?}", fn_name, error);
        format!("{}: RedisError={:?}", fn_name, error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::redis_pool;
    use redis::AsyncCommands;

    const REDIS_URL: &str = "redis://localhost:6379";

    #[actix_rt::test]
    #[ignore]
    async fn integration_get_string_some() -> Result<(), String> {
        let mut pool = redis_pool::new(REDIS_URL).await?;
        let _: () = pool
            .set("integration_test:get_string:some", "value")
            .await
            .map_err(|error| RedisCache::map_error("integration_get_string_some", &error))?;

        let cache = RedisCache::new(&pool);

        assert_eq!(
            cache.get_string("integration_test:get_string:some").await?,
            Some("value".to_owned())
        );

        Ok(())
    }

    #[actix_rt::test]
    #[ignore]
    async fn integration_get_string_none() -> Result<(), String> {
        let pool = redis_pool::new(REDIS_URL).await?;
        let cache = RedisCache::new(&pool);

        assert_eq!(
            cache.get_string("integration_test:get_string:none").await?,
            None
        );

        Ok(())
    }

    #[actix_rt::test]
    #[ignore]
    async fn integration_set_string() -> Result<(), String> {
        let pool = redis_pool::new(REDIS_URL).await?;
        let cache = RedisCache::new(&pool);

        assert_eq!(
            cache
                .set_string("integration_test:set_string", "value")
                .await?,
            ()
        );

        Ok(())
    }
}
