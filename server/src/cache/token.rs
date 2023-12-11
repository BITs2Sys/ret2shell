//! Token cache functions

use redis::RedisError;

use super::manager::{CacheError, PoolLike, PooledConnectionLike, RedisPool};

pub struct Token;

impl Token {
    pub async fn store(
        conn: &mut RedisPool, user_id: i64, token: &str,
    ) -> Result<(), CacheError<RedisError>> {
        let mut conn = conn.get().await?;
        // 86400000 ms == 24 hours
        conn.pset_ex(format!("token:{token}"), user_id, 86400000)
            .await?;
        conn.rpush(format!("token-user:{}", user_id), token).await?;
        Ok(())
    }

    pub async fn validate(conn: &mut RedisPool, token: &str) -> Result<(), CacheError<RedisError>> {
        let mut conn = conn.get().await?;
        let _: i64 = conn.get(format!("token:{token}")).await?;
        Ok(())
    }

    pub async fn revoke(conn: &mut RedisPool, token: &str) -> Result<(), CacheError<RedisError>> {
        let mut conn = conn.get().await?;
        let user_id: i64 = conn.get(format!("token:{token}")).await?;
        conn.del(format!("token:{token}")).await?;
        conn.lrem(format!("token-user:{}", user_id), 0, token)
            .await?;
        Ok(())
    }

    pub async fn delete_all(
        conn: &mut RedisPool, user_id: i64,
    ) -> Result<(), CacheError<RedisError>> {
        let mut conn = conn.get().await?;
        let tokens: Vec<String> = conn.lrange(format!("token:{}", user_id), 0, -1).await?;
        for token in tokens {
            conn.del(format!("token:{token}")).await?;
        }
        conn.del(format!("token-user:{}", user_id)).await?;
        Ok(())
    }
}
