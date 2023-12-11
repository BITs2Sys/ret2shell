//! Platform cache functions

use redis::RedisError;
use sea_orm::DatabaseConnection;

use super::manager::{CacheError, PoolLike, PooledConnectionLike, RedisPool};
use crate::entity::config::{self, Model as PlatformInfoModel};

pub struct Platform;

impl Platform {
    pub async fn refresh_cache(
        conn: &mut RedisPool, db: &DatabaseConnection,
    ) -> Result<PlatformInfoModel, CacheError<RedisError>> {
        let config = config::get_config(db).await?;
        let mut conn = conn.get().await?;
        conn.set("config", serde_json::to_string(&config)?).await?;
        Ok(config)
    }

    pub async fn get(
        pool: &mut RedisPool, db: &DatabaseConnection,
    ) -> Result<PlatformInfoModel, CacheError<RedisError>> {
        let _pool = pool.clone();
        let mut conn = _pool.get().await?;
        let config: Option<String> = conn.get("config").await?;
        if let Some(config) = config {
            Ok(serde_json::from_str(&config)?)
        } else {
            Ok(Self::refresh_cache(pool, db).await?)
        }
    }
}
