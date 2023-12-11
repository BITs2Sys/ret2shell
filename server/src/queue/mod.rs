//! Provides message queue for other modules.

use async_nats::jetstream;

use crate::config::GlobalConfig;

pub async fn initialize(config: &GlobalConfig) -> anyhow::Result<jetstream::Context> {
    let mut options =
        async_nats::ConnectOptions::new().require_tls(config.queue.tls.unwrap_or(false));
    if config.queue.token.is_some() {
        options = options.token(
            config
                .queue
                .token
                .clone()
                .unwrap_or("IMPOSSIBLE".to_owned()),
        );
    } else if config.queue.user.is_some() && config.queue.password.is_some() {
        options = options.user_and_password(
            config.queue.user.clone().unwrap_or("IMPOSSIBLE".to_owned()),
            config
                .queue
                .password
                .clone()
                .unwrap_or("IMPOSSIBLE".to_owned()),
        )
    }
    let client = options.connect(config.queue.addr()).await?;
    Ok(jetstream::new(client))
}
