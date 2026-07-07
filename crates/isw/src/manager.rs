//! `IswManager` — builds per-host [`AgentClient`]s for the platform.
//!
//! When `R2S_ISW_CA`/`CERT`/`KEY` are set the client speaks mTLS over https: it presents
//! its client identity and validates the agent's cert against the shared CA **and** the
//! standard TLS hostname (SAN) check — reqwest verifies the presented cert's SAN matches
//! the connection host (`host.address`), so a host-agent whose cert is scoped to its own
//! address cannot impersonate another host. Agent certs MUST therefore be issued
//! address-scoped (SAN = the host's address); a future hardening will additionally pin
//! `isw_host.fingerprint` for defence-in-depth. Without the TLS env vars it falls back to
//! plain http (trusted-LAN / dev).

use std::time::Duration;

use crate::{IswError, client::AgentClient};

#[derive(Clone)]
pub struct IswManager {
  http: reqwest::Client,
  /// shared bearer token presented to every host-agent (per-host tokens later).
  token: String,
  /// http vs https scheme for agent base URLs (https once mTLS lands).
  scheme: &'static str,
}

impl IswManager {
  /// Build a manager. Reads the shared agent bearer token from `R2S_ISW_TOKEN`.
  /// If `R2S_ISW_CA` + `R2S_ISW_CERT` + `R2S_ISW_KEY` (PEM file paths) are all set,
  /// the client speaks **mTLS** (rustls, client identity, CA validation + TLS hostname
  /// verification against the target host address) over https; otherwise it falls back
  /// to plain http (trusted-LAN / dev).
  pub fn initialize() -> Result<Self, IswError> {
    let token = std::env::var("R2S_ISW_TOKEN").unwrap_or_default();
    let mut builder = reqwest::Client::builder()
      .connect_timeout(Duration::from_secs(5))
      .timeout(Duration::from_secs(120));

    let tls = (
      std::env::var("R2S_ISW_CA"),
      std::env::var("R2S_ISW_CERT"),
      std::env::var("R2S_ISW_KEY"),
    );
    let scheme = if let (Ok(ca), Ok(cert), Ok(key)) = tls {
      let ca_pem = std::fs::read(&ca)
        .map_err(|e| IswError::Config(format!("read R2S_ISW_CA {ca}: {e}")))?;
      let root = reqwest::Certificate::from_pem(&ca_pem)?;
      // reqwest's Identity::from_pem wants the client cert and key concatenated.
      let mut identity_pem = std::fs::read(&cert)
        .map_err(|e| IswError::Config(format!("read R2S_ISW_CERT {cert}: {e}")))?;
      identity_pem.push(b'\n');
      identity_pem.extend_from_slice(
        &std::fs::read(&key)
          .map_err(|e| IswError::Config(format!("read R2S_ISW_KEY {key}: {e}")))?,
      );
      let identity = reqwest::Identity::from_pem(&identity_pem)?;
      builder = builder
        .use_rustls_tls()
        .add_root_certificate(root)
        .identity(identity)
        .https_only(true);
      "https"
    } else {
      "http"
    };

    let http = builder.build().map_err(IswError::Http)?;
    Ok(Self { http, token, scheme })
  }

  /// A client for the host reachable at `address:port`.
  pub fn client_for(&self, address: &str, port: i32) -> AgentClient {
    let base = format!("{}://{}:{}", self.scheme, address, port);
    AgentClient::new(self.http.clone(), base, self.token.clone())
  }
}
