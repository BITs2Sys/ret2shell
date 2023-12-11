//! Provides dynamic challenge container service management for players.
//!
//! backend using kube and k3s.

use kube::{
    config::{KubeConfigOptions, Kubeconfig},
    Client, Config,
};

use crate::config::GlobalConfig;

pub async fn initialize(config: &GlobalConfig) -> anyhow::Result<Client> {
    let client = if config.cluster.try_default {
        Client::try_default().await?
    } else if config.cluster.auto_infer {
        Client::try_from(Config::infer().await?)?
    } else {
        let kube_config = Kubeconfig::read_from(config.cluster.kube_config_path.as_ref().unwrap())?;
        let kube_config =
            Config::from_custom_kubeconfig(kube_config, &KubeConfigOptions::default()).await?;
        Client::try_from(kube_config)?
    };
    Ok(client)
}
