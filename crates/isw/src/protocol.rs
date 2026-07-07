//! Wire protocol shared between the platform (`r2s-isw` client) and the per-host
//! `r2s-isw-agent`. HTTP/JSON over mTLS (mTLS wired in a later phase).

use serde::{Deserialize, Serialize};

/// Runtime state of one guest VM as reported by a host-agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmState {
  pub logical_name: String,
  /// "on" | "off" | "unknown".
  pub power_state: String,
  /// "running" | "not_running" | "unknown".
  pub tools_state: String,
  pub ip: Option<String>,
}

/// `GET /v1/health` response — heartbeat used by the platform scheduler.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
  pub free_mem_mb: i64,
  pub vmrun_ok: bool,
  pub vms: Vec<VmState>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PowerOp {
  Start,
  StopSoft,
  StopHard,
  Reset,
  Suspend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerRequest {
  pub op: PowerOp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotRequest {
  pub name: String,
}

/// `POST /v1/vms/{vm}/inject` — push a flag file into a guest and set perms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectRequest {
  pub guest_path: String,
  /// base64 of the flag bytes (never sent as a plaintext command argument).
  pub content_b64: String,
  /// guest owner spec (linux `user:group`, or a windows user for icacls).
  pub owner: Option<String>,
  /// guest file mode (octal string, e.g. "0640").
  pub mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectResult {
  pub ok: bool,
  /// sha256 read back from the guest after injection (for verification).
  pub sha256: Option<String>,
  pub message: Option<String>,
}

/// `POST /v1/vms/{vm}/run` — run a program or a script in the guest.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunRequest {
  /// interpreter for a script body, e.g. "/bin/bash" or "cmd.exe".
  pub interpreter: Option<String>,
  /// script body (used with `interpreter`).
  pub script: Option<String>,
  /// program path (mutually exclusive with `script`).
  pub program: Option<String>,
  #[serde(default)]
  pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
  pub exit_code: i32,
  /// base64 of captured stdout (captured via guest-file read-back).
  pub stdout_b64: Option<String>,
  pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestIp {
  pub ip: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyRequest {
  pub guest_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResult {
  pub exists: bool,
  pub sha256: Option<String>,
}

/// Generic ok/message result for power/snapshot/revert ops.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpResult {
  pub ok: bool,
  pub message: Option<String>,
}

/// `POST /v1/vpn/peer` — provision a WireGuard peer for a team on this host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnPeerRequest {
  /// address to assign the peer inside the range subnet, e.g. "10.50.1.11".
  pub address: String,
  /// allowed-ips the client routes into the range, e.g. "10.50.1.0/24".
  pub subnet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnPeerResponse {
  pub client_private_key: String,
  pub client_public_key: String,
  pub server_public_key: String,
  /// public endpoint the client dials, e.g. "host-a:51820".
  pub endpoint: String,
  pub dns: Option<String>,
}
