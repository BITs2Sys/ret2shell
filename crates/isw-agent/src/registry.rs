//! Agent configuration + VM registry (loaded from a local TOML file).
//!
//! Guest credentials live here, on the host, and are never sent by the platform.

use std::{collections::HashMap, path::Path};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct VmEntry {
  pub logical_name: String,
  /// absolute `.vmx` path (or relative to `range_root`).
  pub vmx: String,
  /// "linux" | "windows".
  #[serde(default = "default_os")]
  pub guest_os: String,
  pub guest_user: String,
  pub guest_pass: String,
}

fn default_os() -> String {
  "linux".to_owned()
}

/// Optional mTLS: when present the agent serves HTTPS and requires a client
/// certificate signed by `ca`. When absent it serves plain HTTP (dev / trusted LAN).
#[derive(Debug, Clone, Deserialize)]
pub struct TlsConfig {
  /// PEM server certificate (chain).
  pub cert: String,
  /// PEM server private key (PKCS#8).
  pub key: String,
  /// PEM CA used to verify the platform's client certificate.
  pub ca: String,
}

/// WireGuard settings for brokering team access into this host's range network.
#[derive(Debug, Clone, Deserialize)]
pub struct VpnConfig {
  /// wg interface name on the host, e.g. "wg0".
  pub interface: String,
  /// public endpoint teams dial, e.g. "host-a.example:51820".
  pub endpoint: String,
  /// the host wg interface's public key.
  pub server_public_key: String,
  #[serde(default)]
  pub dns: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
  /// listen address, e.g. "0.0.0.0:8443".
  #[serde(default = "default_listen")]
  pub listen: String,
  /// shared bearer token the platform must present.
  pub token: String,
  /// optional explicit vmrun path (autodetected when absent).
  #[serde(default)]
  pub vmrun_path: Option<String>,
  /// optional root prefix for relative `.vmx` paths.
  #[serde(default)]
  pub range_root: Option<String>,
  /// optional mTLS; plain HTTP when omitted.
  #[serde(default)]
  pub tls: Option<TlsConfig>,
  /// optional WireGuard brokering for team range access.
  #[serde(default)]
  pub vpn: Option<VpnConfig>,
  #[serde(default, rename = "vm")]
  pub vms: Vec<VmEntry>,
}

fn default_listen() -> String {
  "0.0.0.0:8443".to_owned()
}

impl AgentConfig {
  pub async fn load(path: &Path) -> Result<Self> {
    let raw = tokio::fs::read_to_string(path)
      .await
      .with_context(|| format!("failed to read agent config {}", path.display()))?;
    let cfg: AgentConfig = toml::from_str(&raw).context("failed to parse agent config toml")?;
    Ok(cfg)
  }
}

/// Fast lookup of VM entries by logical name.
#[derive(Debug, Clone)]
pub struct Registry {
  by_name: HashMap<String, VmEntry>,
  range_root: Option<String>,
}

impl Registry {
  pub fn new(vms: Vec<VmEntry>, range_root: Option<String>) -> Self {
    let by_name = vms
      .into_iter()
      .map(|v| (v.logical_name.clone(), v))
      .collect();
    Self { by_name, range_root }
  }

  pub fn get(&self, logical_name: &str) -> Option<&VmEntry> {
    self.by_name.get(logical_name)
  }

  pub fn names(&self) -> Vec<String> {
    let mut names: Vec<String> = self.by_name.keys().cloned().collect();
    names.sort();
    names
  }

  /// Resolve a possibly-relative `.vmx` path against the configured range root.
  pub fn resolve_vmx(&self, entry: &VmEntry) -> String {
    let p = Path::new(&entry.vmx);
    if p.is_absolute() {
      return entry.vmx.clone();
    }
    match &self.range_root {
      Some(root) => Path::new(root)
        .join(&entry.vmx)
        .to_string_lossy()
        .into_owned(),
      None => entry.vmx.clone(),
    }
  }
}
