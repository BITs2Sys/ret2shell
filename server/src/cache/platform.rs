//! Platform cache functions
//!

use redis::RedisError;
use sea_orm::DatabaseConnection;

use crate::entity::config::{self, Model as PlatformInfoModel};

use super::manager::{CacheError, PoolLike, PooledConnectionLike, RedisPool};

pub struct Platform;

impl Platform {
    pub async fn refresh_cache(
        conn: &mut RedisPool,
        db: &DatabaseConnection,
    ) -> Result<PlatformInfoModel, CacheError<RedisError>> {
        let platform_info = config::get_config(db).await?;
        let mut conn = conn.get().await?;
        conn.set("platform_info", serde_json::to_string(&platform_info)?)
            .await?;
        Ok(platform_info)
    }

    pub async fn get(
        pool: &mut RedisPool,
        db: &DatabaseConnection,
    ) -> Result<PlatformInfoModel, CacheError<RedisError>> {
        let _pool = pool.clone();
        let mut conn = _pool.get().await?;
        let platform_info: Option<String> = conn.get("platform_info").await?;
        if let Some(platform_info) = platform_info {
            Ok(serde_json::from_str(&platform_info)?)
        } else {
            Ok(Self::refresh_cache(pool, db).await?)
        }
    }
}
