//! WireGuard peer provisioning: generate a client keypair with `wg` and add the
//! peer to the host's running interface. The platform turns the response into a
//! downloadable client config for the team.

use anyhow::{Context, Result, anyhow};
use r2s_isw::protocol::{VpnPeerRequest, VpnPeerResponse};
use tokio::{io::AsyncWriteExt, process::Command};

use crate::registry::VpnConfig;

async fn wg_genkey() -> Result<String> {
  let out = Command::new("wg")
    .arg("genkey")
    .output()
    .await
    .context("run `wg genkey`")?;
  if !out.status.success() {
    return Err(anyhow!("wg genkey failed"));
  }
  Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

async fn wg_pubkey(private: &str) -> Result<String> {
  let mut child = Command::new("wg")
    .arg("pubkey")
    .stdin(std::process::Stdio::piped())
    .stdout(std::process::Stdio::piped())
    .spawn()
    .context("spawn `wg pubkey`")?;
  child
    .stdin
    .take()
    .ok_or_else(|| anyhow!("no stdin for wg pubkey"))?
    .write_all(format!("{private}\n").as_bytes())
    .await?;
  let out = child.wait_with_output().await?;
  if !out.status.success() {
    return Err(anyhow!("wg pubkey failed"));
  }
  Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

/// Generate a client keypair and register it as a peer on the host interface,
/// scoping its allowed-ips to the assigned address.
pub async fn provision_peer(cfg: &VpnConfig, req: &VpnPeerRequest) -> Result<VpnPeerResponse> {
  let private = wg_genkey().await?;
  let public = wg_pubkey(&private).await?;
  let status = Command::new("wg")
    .args([
      "set",
      &cfg.interface,
      "peer",
      &public,
      "allowed-ips",
      &format!("{}/32", req.address),
    ])
    .status()
    .await
    .context("`wg set` peer")?;
  if !status.success() {
    return Err(anyhow!("wg set failed for interface {}", cfg.interface));
  }
  // `wg set` is runtime-only; persist the peer to the interface config so it survives an
  // interface restart / host reboot. Best-effort — a save failure must not fail
  // provisioning (the peer is already live).
  let _ = Command::new("wg-quick")
    .args(["save", &cfg.interface])
    .status()
    .await;
  Ok(VpnPeerResponse {
    client_private_key: private,
    client_public_key: public,
    server_public_key: cfg.server_public_key.clone(),
    endpoint: cfg.endpoint.clone(),
    dns: cfg.dns.clone(),
  })
}
