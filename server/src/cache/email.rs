use redis::RedisError;

use super::manager::{CacheError, PoolLike, PooledConnectionLike, RedisPool};

pub struct Email;

impl Email {
    pub async fn add_validation(
        conn: &mut RedisPool, id: &str, email: &str,
    ) -> Result<(), CacheError<RedisError>> {
        let mut conn = conn.get().await?;
        conn.pset_ex(format!("email:{}", id), email, 30 * 60 * 1000)
            .await?;
        Ok(())
    }

    pub async fn check_validation(
        conn: &mut RedisPool, id: &str,
    ) -> Result<Option<String>, CacheError<RedisError>> {
        let mut conn = conn.get().await?;
        let email: Option<String> = conn.get(format!("email:{}", id)).await?;
        if let Some(email) = email {
            conn.del(format!("email:{}", id)).await?;
            Ok(Some(email))
        } else {
            Ok(None)
        }
    }

    pub async fn add_freq_limit(
        conn: &mut RedisPool, email: &str,
    ) -> Result<(), CacheError<RedisError>> {
        let mut conn = conn.get().await?;
        let cnt: Option<i32> = conn.get(email).await?;
        match cnt {
            Some(cnt) => {
                conn.set(email, cnt + 1).await?;
            }
            None => {
                conn.pset_ex(email, 1, 10 * 60 * 1000).await?;
            }
        }
        Ok(())
    }

    pub async fn check_freq_limit(
        conn: &mut RedisPool, email: &str,
    ) -> Result<bool, CacheError<RedisError>> {
        let mut conn = conn.get().await?;
        let cnt: Option<i32> = conn.get(email).await?;
        match cnt {
            Some(cnt) => Ok(cnt <= 3),
            None => Ok(true),
        }
    }
}
