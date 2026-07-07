//! HTTP/JSON client the platform uses to drive a single host-agent.

use serde::{Serialize, de::DeserializeOwned};

use crate::{
  IswError,
  protocol::{
    GuestIp, HealthResponse, InjectRequest, InjectResult, OpResult, PowerOp, PowerRequest,
    RunRequest, RunResult, SnapshotRequest, VerifyRequest, VerifyResult, VmState, VpnPeerRequest,
    VpnPeerResponse,
  },
};

/// A typed client bound to one host-agent base URL + bearer token.
#[derive(Clone)]
pub struct AgentClient {
  http: reqwest::Client,
  base: String,
  token: String,
}

impl AgentClient {
  pub fn new(http: reqwest::Client, base: impl Into<String>, token: impl Into<String>) -> Self {
    Self {
      http,
      base: base.into().trim_end_matches('/').to_owned(),
      token: token.into(),
    }
  }

  fn url(&self, path: &str) -> String {
    format!("{}{}", self.base, path)
  }

  async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, IswError> {
    let resp = self
      .http
      .get(self.url(path))
      .bearer_auth(&self.token)
      .send()
      .await?;
    Self::parse(resp).await
  }

  async fn post<B: Serialize, T: DeserializeOwned>(
    &self, path: &str, body: &B,
  ) -> Result<T, IswError> {
    let resp = self
      .http
      .post(self.url(path))
      .bearer_auth(&self.token)
      .json(body)
      .send()
      .await?;
    Self::parse(resp).await
  }

  async fn parse<T: DeserializeOwned>(resp: reqwest::Response) -> Result<T, IswError> {
    let status = resp.status();
    if !status.is_success() {
      let body = resp.text().await.unwrap_or_default();
      return Err(IswError::Agent(format!("agent returned {status}: {body}")));
    }
    Ok(resp.json::<T>().await?)
  }

  pub async fn health(&self) -> Result<HealthResponse, IswError> {
    self.get("/v1/health").await
  }

  pub async fn list_vms(&self) -> Result<Vec<VmState>, IswError> {
    self.get("/v1/vms").await
  }

  pub async fn power(&self, vm: &str, op: PowerOp) -> Result<OpResult, IswError> {
    self
      .post(&format!("/v1/vms/{}/power", enc(vm)), &PowerRequest { op })
      .await
  }

  pub async fn snapshot(&self, vm: &str, name: &str) -> Result<OpResult, IswError> {
    self
      .post(
        &format!("/v1/vms/{}/snapshot", enc(vm)),
        &SnapshotRequest {
          name: name.to_owned(),
        },
      )
      .await
  }

  pub async fn revert(&self, vm: &str, name: &str) -> Result<OpResult, IswError> {
    self
      .post(
        &format!("/v1/vms/{}/revert", enc(vm)),
        &SnapshotRequest {
          name: name.to_owned(),
        },
      )
      .await
  }

  pub async fn inject(&self, vm: &str, req: &InjectRequest) -> Result<InjectResult, IswError> {
    self
      .post(&format!("/v1/vms/{}/inject", enc(vm)), req)
      .await
  }

  pub async fn run(&self, vm: &str, req: &RunRequest) -> Result<RunResult, IswError> {
    self.post(&format!("/v1/vms/{}/run", enc(vm)), req).await
  }

  pub async fn guest_ip(&self, vm: &str) -> Result<GuestIp, IswError> {
    self.get(&format!("/v1/vms/{}/ip", enc(vm))).await
  }

  pub async fn verify(&self, vm: &str, req: &VerifyRequest) -> Result<VerifyResult, IswError> {
    self
      .post(&format!("/v1/vms/{}/verify", enc(vm)), req)
      .await
  }

  pub async fn provision_vpn(&self, req: &VpnPeerRequest) -> Result<VpnPeerResponse, IswError> {
    self.post("/v1/vpn/peer", req).await
  }
}

/// Minimal path-segment encoder (logical VM names are `[A-Za-z0-9_-]`, but be safe).
fn enc(segment: &str) -> String {
  segment
    .chars()
    .map(|c| {
      if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
        c.to_string()
      } else {
        format!("%{:02X}", c as u32)
      }
    })
    .collect()
}
