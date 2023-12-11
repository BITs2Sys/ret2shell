//! The in-memory cache provider, which is used to store the temporary data like
//! user's token.
//!
//! The cache provider is implemented by using the `fred` crate and `redis`.

use crate::config::GlobalConfig;

pub mod captcha;
pub mod cluster;
pub mod email;
pub mod manager;
pub mod platform;
pub mod token;

pub use captcha::Captcha;
pub use email::Email;
pub use token::Token;

/// Init the cache manager.
pub async fn initialize(config: &GlobalConfig) -> anyhow::Result<manager::RedisPool> {
    let nodes = &config.cache.nodes;
    let max_connections = config.cache.max_connections.unwrap_or(32);
    let mgr = manager::new_redis_pool(nodes, max_connections).await?;
    Ok(mgr)
}
