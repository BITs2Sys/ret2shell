//! `r2s-isw-agent` — per-host ISW range agent for the BITs2CTF fork of Ret2Shell.
//!
//! Exposes an HTTP/JSON API the platform drives to power/snapshot/revert VMs and to
//! inject + verify flags inside guests via `vmrun`. Guest credentials stay local.
//!
//! Phase 1: plain HTTP + shared bearer token. Phase 2 adds mTLS (rustls) and a
//! pinned server certificate.

mod registry;
mod tls;
mod vmrun;
mod wg;

use std::{collections::HashMap, path::Path, sync::Arc};

use anyhow::{Context, Result};
use axum::{
  Json, Router,
  extract::{Path as AxPath, Request, State},
  http::{StatusCode, header::AUTHORIZATION},
  middleware::Next,
  response::{IntoResponse, Response},
  routing::{get, post},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use clap::Parser;
use r2s_isw::protocol::{
  GuestIp, HealthResponse, InjectRequest, InjectResult, OpResult, PowerRequest, RunRequest,
  RunResult, SnapshotRequest, VerifyRequest, VerifyResult, VmState, VpnPeerRequest, VpnPeerResponse,
};
use registry::{AgentConfig, Registry, VmEntry, VpnConfig};
use tokio::sync::Mutex;
use tracing_subscriber::EnvFilter;
use vmrun::Vmrun;

#[derive(Parser)]
#[command(name = "r2s-isw-agent", about = "Ret2Shell ISW per-host range agent")]
struct Cli {
  /// path to the agent TOML config (token, vmrun path, VM registry).
  #[arg(short, long, default_value = "agent.toml")]
  config: String,
}

#[derive(Clone)]
struct AppState {
  vmrun: Arc<Vmrun>,
  registry: Arc<Registry>,
  token: Arc<String>,
  vpn: Option<Arc<VpnConfig>>,
  locks: Arc<Mutex<HashMap<String, Arc<Mutex<()>>>>>,
}

impl AppState {
  /// A per-VM mutex so guest operations on one VM never overlap (VIX dislikes it).
  async fn vm_lock(&self, name: &str) -> Arc<Mutex<()>> {
    let mut locks = self.locks.lock().await;
    locks
      .entry(name.to_owned())
      .or_insert_with(|| Arc::new(Mutex::new(())))
      .clone()
  }
}

#[tokio::main]
async fn main() -> Result<()> {
  tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
    .init();

  let cli = Cli::parse();
  let cfg = AgentConfig::load(Path::new(&cli.config)).await?;
  let listen = cfg.listen.clone();
  let tls_cfg = cfg.tls.clone();
  let vmrun = Vmrun::detect(cfg.vmrun_path.as_deref())?;
  tracing::info!(vmrun = %vmrun.path().display(), vms = cfg.vms.len(), "isw-agent starting");
  let registry = Registry::new(cfg.vms, cfg.range_root);

  let state = AppState {
    vmrun: Arc::new(vmrun),
    registry: Arc::new(registry),
    token: Arc::new(cfg.token),
    vpn: cfg.vpn.map(Arc::new),
    locks: Arc::new(Mutex::new(HashMap::new())),
  };

  let app = Router::new()
    .route("/v1/health", get(health))
    .route("/v1/vms", get(list_vms))
    .route("/v1/vms/{vm}/power", post(power))
    .route("/v1/vms/{vm}/snapshot", post(snapshot))
    .route("/v1/vms/{vm}/revert", post(revert))
    .route("/v1/vms/{vm}/inject", post(inject))
    .route("/v1/vms/{vm}/run", post(run))
    .route("/v1/vms/{vm}/ip", get(guest_ip))
    .route("/v1/vms/{vm}/verify", post(verify))
    .route("/v1/vpn/peer", post(provision_vpn))
    .layer(axum::middleware::from_fn_with_state(state.clone(), auth))
    .with_state(state);

  match tls_cfg {
    Some(tls) => tls::serve(&listen, app, &tls).await?,
    None => {
      let listener = tokio::net::TcpListener::bind(&listen)
        .await
        .with_context(|| format!("failed to bind {listen}"))?;
      tracing::info!(%listen, "isw-agent listening (http)");
      axum::serve(listener, app).await?;
    }
  }
  Ok(())
}

/// Bearer-token gate. If no token is configured, the agent runs open (dev only).
async fn auth(State(state): State<AppState>, req: Request, next: Next) -> Result<Response, StatusCode> {
  if state.token.is_empty() {
    return Ok(next.run(req).await);
  }
  let ok = req
    .headers()
    .get(AUTHORIZATION)
    .and_then(|v| v.to_str().ok())
    .and_then(|v| v.strip_prefix("Bearer "))
    .map(|t| t == state.token.as_str())
    .unwrap_or(false);
  if ok {
    Ok(next.run(req).await)
  } else {
    Err(StatusCode::UNAUTHORIZED)
  }
}

// ---- error type -----------------------------------------------------------

struct AppError {
  code: StatusCode,
  msg: String,
}

impl AppError {
  fn new(code: StatusCode, msg: impl Into<String>) -> Self {
    Self {
      code,
      msg: msg.into(),
    }
  }
}

impl From<anyhow::Error> for AppError {
  fn from(e: anyhow::Error) -> Self {
    Self::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
  }
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    (self.code, self.msg).into_response()
  }
}

fn resolve_vm<'a>(state: &'a AppState, name: &str) -> Result<&'a VmEntry, AppError> {
  state
    .registry
    .get(name)
    .ok_or_else(|| AppError::new(StatusCode::NOT_FOUND, format!("unknown vm: {name}")))
}

fn sha256_hex(bytes: &[u8]) -> String {
  ring::digest::digest(&ring::digest::SHA256, bytes)
    .as_ref()
    .iter()
    .fold(String::new(), |mut acc, b| {
      acc.push_str(&format!("{b:02x}"));
      acc
    })
}

#[cfg(target_os = "linux")]
fn free_mem_mb() -> i64 {
  std::fs::read_to_string("/proc/meminfo")
    .ok()
    .and_then(|s| {
      s.lines()
        .find_map(|l| l.strip_prefix("MemAvailable:"))
        .and_then(|rest| rest.split_whitespace().next().map(str::to_owned))
    })
    .and_then(|kb| kb.parse::<i64>().ok())
    .map(|kb| kb / 1024)
    .unwrap_or(-1)
}

#[cfg(not(target_os = "linux"))]
fn free_mem_mb() -> i64 {
  // TODO(phase-2): query Windows GlobalMemoryStatusEx via a host API.
  -1
}

// ---- handlers -------------------------------------------------------------

async fn health(State(state): State<AppState>) -> Result<Json<HealthResponse>, AppError> {
  let running = state.vmrun.list_running().await.unwrap_or_default();
  let vms = state
    .registry
    .names()
    .into_iter()
    .filter_map(|name| {
      let entry = state.registry.get(&name)?;
      let vmx = state.registry.resolve_vmx(entry);
      let power = if running.iter().any(|r| r == &vmx) {
        "on"
      } else {
        "off"
      };
      Some(VmState {
        logical_name: name,
        power_state: power.to_owned(),
        tools_state: "unknown".to_owned(),
        ip: None,
      })
    })
    .collect();
  Ok(Json(HealthResponse {
    free_mem_mb: free_mem_mb(),
    vmrun_ok: state.vmrun.path().exists() || state.vmrun.path().to_string_lossy() == "vmrun",
    vms,
  }))
}

async fn list_vms(State(state): State<AppState>) -> Result<Json<Vec<VmState>>, AppError> {
  let health = health(State(state)).await?;
  Ok(Json(health.0.vms))
}

async fn power(
  State(state): State<AppState>, AxPath(vm): AxPath<String>, Json(req): Json<PowerRequest>,
) -> Result<Json<OpResult>, AppError> {
  let entry = resolve_vm(&state, &vm)?.clone();
  let vmx = state.registry.resolve_vmx(&entry);
  let lock = state.vm_lock(&vm).await;
  let _g = lock.lock().await;
  let out = state.vmrun.power(&vmx, req.op).await.map_err(AppError::from)?;
  Ok(Json(OpResult {
    ok: out.ok(),
    message: Some(out.combined()),
  }))
}

async fn snapshot(
  State(state): State<AppState>, AxPath(vm): AxPath<String>, Json(req): Json<SnapshotRequest>,
) -> Result<Json<OpResult>, AppError> {
  let entry = resolve_vm(&state, &vm)?.clone();
  let vmx = state.registry.resolve_vmx(&entry);
  let lock = state.vm_lock(&vm).await;
  let _g = lock.lock().await;
  let out = state
    .vmrun
    .snapshot(&vmx, &req.name)
    .await
    .map_err(AppError::from)?;
  Ok(Json(OpResult {
    ok: out.ok(),
    message: Some(out.combined()),
  }))
}

async fn revert(
  State(state): State<AppState>, AxPath(vm): AxPath<String>, Json(req): Json<SnapshotRequest>,
) -> Result<Json<OpResult>, AppError> {
  let entry = resolve_vm(&state, &vm)?.clone();
  let vmx = state.registry.resolve_vmx(&entry);
  let lock = state.vm_lock(&vm).await;
  let _g = lock.lock().await;
  let out = state
    .vmrun
    .revert(&vmx, &req.name)
    .await
    .map_err(AppError::from)?;
  Ok(Json(OpResult {
    ok: out.ok(),
    message: Some(out.combined()),
  }))
}

async fn inject(
  State(state): State<AppState>, AxPath(vm): AxPath<String>, Json(req): Json<InjectRequest>,
) -> Result<Json<InjectResult>, AppError> {
  let entry = resolve_vm(&state, &vm)?.clone();
  let lock = state.vm_lock(&vm).await;
  let _g = lock.lock().await;
  let res = do_inject(&state, &entry, &req).await.map_err(AppError::from)?;
  Ok(Json(res))
}

async fn do_inject(
  state: &AppState, entry: &VmEntry, req: &InjectRequest,
) -> Result<InjectResult> {
  let vmx = state.registry.resolve_vmx(entry);
  let bytes = STANDARD
    .decode(req.content_b64.as_bytes())
    .context("invalid base64 content")?;
  let expected = sha256_hex(&bytes);

  // stage on host, keyed by content hash so it is unique + idempotent.
  let stage = std::env::temp_dir().join(format!("r2s-isw-stage-{expected}"));
  tokio::fs::write(&stage, &bytes)
    .await
    .context("failed to stage flag on host")?;

  let push = state
    .vmrun
    .copy_to_guest(
      &vmx,
      &entry.guest_user,
      &entry.guest_pass,
      &stage.to_string_lossy(),
      &req.guest_path,
    )
    .await?;
  if !push.ok() {
    let _ = tokio::fs::remove_file(&stage).await;
    return Ok(InjectResult {
      ok: false,
      sha256: None,
      message: Some(format!("copyFileFromHostToGuest failed: {}", push.combined())),
    });
  }

  // set ownership / mode.
  apply_perms(state, entry, &vmx, req).await?;

  // verify by read-back hash.
  let verify_local = std::env::temp_dir().join(format!("r2s-isw-verify-{expected}"));
  let pull = state
    .vmrun
    .copy_from_guest(
      &vmx,
      &entry.guest_user,
      &entry.guest_pass,
      &req.guest_path,
      &verify_local.to_string_lossy(),
    )
    .await?;
  let mut sha = None;
  let mut ok = false;
  if pull.ok()
    && let Ok(read_back) = tokio::fs::read(&verify_local).await
  {
    let got = sha256_hex(&read_back);
    ok = got == expected;
    sha = Some(got);
  }

  let _ = tokio::fs::remove_file(&stage).await;
  let _ = tokio::fs::remove_file(&verify_local).await;

  Ok(InjectResult {
    ok,
    sha256: sha,
    message: if ok {
      None
    } else {
      Some("read-back hash mismatch or copy-back failed".to_owned())
    },
  })
}

/// Apply guest ownership/mode after injection (best effort; guest-OS specific).
async fn apply_perms(
  state: &AppState, entry: &VmEntry, vmx: &str, req: &InjectRequest,
) -> Result<()> {
  let is_windows = entry.guest_os.eq_ignore_ascii_case("windows");
  if is_windows {
    // grant read to the target user via icacls; skip if no owner given.
    if let Some(owner) = &req.owner {
      let cmd = format!(
        "icacls \"{}\" /inheritance:r /grant \"{}\":(R)",
        req.guest_path, owner
      );
      let _ = state
        .vmrun
        .run_program_in_guest(
          vmx,
          &entry.guest_user,
          &entry.guest_pass,
          "C:\\Windows\\System32\\cmd.exe",
          &["/c".to_owned(), cmd],
        )
        .await?;
    }
    Ok(())
  } else {
    let mut script = String::new();
    if let Some(owner) = &req.owner {
      script.push_str(&format!("chown '{}' '{}'", owner, req.guest_path));
    }
    if let Some(mode) = &req.mode {
      if !script.is_empty() {
        script.push_str(" && ");
      }
      script.push_str(&format!("chmod '{}' '{}'", mode, req.guest_path));
    }
    if !script.is_empty() {
      let _ = state
        .vmrun
        .run_script_in_guest(vmx, &entry.guest_user, &entry.guest_pass, "/bin/bash", &script)
        .await?;
    }
    Ok(())
  }
}

async fn run(
  State(state): State<AppState>, AxPath(vm): AxPath<String>, Json(req): Json<RunRequest>,
) -> Result<Json<RunResult>, AppError> {
  let entry = resolve_vm(&state, &vm)?.clone();
  let vmx = state.registry.resolve_vmx(&entry);
  let lock = state.vm_lock(&vm).await;
  let _g = lock.lock().await;
  let out = if let (Some(interp), Some(script)) = (&req.interpreter, &req.script) {
    state
      .vmrun
      .run_script_in_guest(&vmx, &entry.guest_user, &entry.guest_pass, interp, script)
      .await
  } else if let Some(program) = &req.program {
    state
      .vmrun
      .run_program_in_guest(&vmx, &entry.guest_user, &entry.guest_pass, program, &req.args)
      .await
  } else {
    return Err(AppError::new(
      StatusCode::BAD_REQUEST,
      "run requires either interpreter+script or program",
    ));
  };
  let out = out.map_err(AppError::from)?;
  Ok(Json(RunResult {
    exit_code: out.code,
    stdout_b64: Some(STANDARD.encode(out.combined().as_bytes())),
    message: None,
  }))
}

async fn guest_ip(
  State(state): State<AppState>, AxPath(vm): AxPath<String>,
) -> Result<Json<GuestIp>, AppError> {
  let entry = resolve_vm(&state, &vm)?.clone();
  let vmx = state.registry.resolve_vmx(&entry);
  let ip = state.vmrun.guest_ip(&vmx).await.map_err(AppError::from)?;
  Ok(Json(GuestIp { ip }))
}

async fn provision_vpn(
  State(state): State<AppState>, Json(req): Json<VpnPeerRequest>,
) -> Result<Json<VpnPeerResponse>, AppError> {
  let cfg = state.vpn.as_ref().ok_or_else(|| {
    AppError::new(
      StatusCode::PRECONDITION_FAILED,
      "vpn is not configured on this host",
    )
  })?;
  let resp = wg::provision_peer(cfg, &req).await.map_err(AppError::from)?;
  Ok(Json(resp))
}

async fn verify(
  State(state): State<AppState>, AxPath(vm): AxPath<String>, Json(req): Json<VerifyRequest>,
) -> Result<Json<VerifyResult>, AppError> {
  let entry = resolve_vm(&state, &vm)?.clone();
  let vmx = state.registry.resolve_vmx(&entry);
  let lock = state.vm_lock(&vm).await;
  let _g = lock.lock().await;
  let exists = state
    .vmrun
    .file_exists_in_guest(&vmx, &entry.guest_user, &entry.guest_pass, &req.guest_path)
    .await
    .map_err(AppError::from)?;
  let mut sha = None;
  if exists {
    let local = std::env::temp_dir().join(format!("r2s-isw-verify-{}", sha256_hex(vm.as_bytes())));
    let pull = state
      .vmrun
      .copy_from_guest(
        &vmx,
        &entry.guest_user,
        &entry.guest_pass,
        &req.guest_path,
        &local.to_string_lossy(),
      )
      .await
      .map_err(AppError::from)?;
    if pull.ok()
      && let Ok(bytes) = tokio::fs::read(&local).await
    {
      sha = Some(sha256_hex(&bytes));
    }
    let _ = tokio::fs::remove_file(&local).await;
  }
  Ok(Json(VerifyResult { exists, sha256: sha }))
}
