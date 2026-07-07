//! BITs2CTF fork: ISW (Internal Security Warfare) range-mode admin routes.
//!
//! Top-level `/range` subsystem gated by `Permission::DevOps` (same gate as the
//! `/cluster` routes). Manages hosts, range templates, range instances, team
//! assignments, and arming (mint + inject flags into the guests via the agent).

use axum::{
  Extension, Json, Router,
  extract::{Path, Query, State},
  http::StatusCode,
  middleware,
  routing::{delete, get, post},
};
use chrono::Utc;
use r2s_database::{
  game, isw_assignment, isw_flag, isw_host, isw_range, isw_range_template, isw_vm, team,
  user::Permission,
};
use r2s_isw::{IswManager, protocol::HealthResponse};
use r2s_migrator::Database;
use serde::{Deserialize, Serialize};

use crate::{
  middleware::{auth, auth::Token, data::extract_team},
  traits::{GlobalState, ResponseError},
  utility::isw::{self, ArmReport},
};

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
  Router::new()
    .route("/host", get(list_hosts).post(create_host))
    .route("/host/{host}", delete(delete_host))
    .route("/host/{host}/health", get(probe_host))
    .route("/template", get(list_templates).post(create_template))
    .route("/template/{template}", delete(delete_template))
    .route("/instance", post(create_range))
    .route("/instance/{range}", get(get_range).delete(delete_range))
    .route("/instance/{range}/arm", post(arm_range))
    .route("/instance/{range}/snapshot", post(snapshot_range))
    .route("/instance/{range}/reset", post(reset_range))
    .route("/instance/{range}/vpn", post(provision_vpn))
    .route("/assignment", post(create_assignment))
    .route("/assignment/{assignment}", delete(delete_assignment))
    .route_layer(middleware::from_fn(auth::permission_required_all!(
      Permission::DevOps
    )))
}

/// Player-facing range routes, nested under `/game/{game}/range` (inherits the
/// game's access + team middleware). Lets a team download its WireGuard config.
pub fn player_router(_state: &GlobalState) -> Router<GlobalState> {
  Router::new().route("/vpn", get(get_my_vpn))
}

// ---- hosts ----------------------------------------------------------------

#[derive(Deserialize)]
struct CreateHost {
  name: String,
  address: String,
  api_port: i32,
  os: String,
  fingerprint: Option<String>,
  enabled: Option<bool>,
}

async fn list_hosts(
  State(db): State<Database>,
) -> Result<Json<Vec<isw_host::Model>>, ResponseError> {
  Ok(Json(isw_host::list(&db.conn).await?))
}

async fn create_host(
  State(db): State<Database>, Json(body): Json<CreateHost>,
) -> Result<Json<isw_host::Model>, ResponseError> {
  let model = isw_host::Model {
    id: 0,
    created_at: Utc::now(),
    name: body.name,
    address: body.address,
    api_port: body.api_port,
    os: body.os,
    fingerprint: body.fingerprint,
    enabled: body.enabled.unwrap_or(true),
    status: "offline".to_owned(),
    free_mem_mb: None,
    last_heartbeat: None,
  };
  Ok(Json(isw_host::create(&db.conn, model).await?))
}

async fn delete_host(
  State(db): State<Database>, Path(id): Path<i64>,
) -> Result<StatusCode, ResponseError> {
  isw_host::delete(&db.conn, id).await?;
  Ok(StatusCode::NO_CONTENT)
}

/// Probe a host-agent's `/v1/health` and record the heartbeat.
async fn probe_host(
  State(db): State<Database>, State(isw): State<IswManager>, Path(id): Path<i64>,
) -> Result<Json<HealthResponse>, ResponseError> {
  let host = isw_host::get(&db.conn, id)
    .await?
    .ok_or_else(|| ResponseError::NotFound(format!("isw host {id}")))?;
  let client = isw.client_for(&host.address, host.api_port);
  let health = client
    .health()
    .await
    .map_err(|e| ResponseError::InternalServerError(e.to_string()))?;
  isw_host::touch_heartbeat(&db.conn, id, "online", Some(health.free_mem_mb)).await?;
  Ok(Json(health))
}

// ---- range templates ------------------------------------------------------

#[derive(Deserialize)]
struct GameQuery {
  game_id: i64,
}

#[derive(Deserialize)]
struct CreateTemplate {
  game_id: i64,
  name: String,
  #[serde(default)]
  brief: String,
  topology: isw_range_template::Topology,
}

async fn list_templates(
  State(db): State<Database>, Query(q): Query<GameQuery>,
) -> Result<Json<Vec<isw_range_template::Model>>, ResponseError> {
  Ok(Json(isw_range_template::list_by_game(&db.conn, q.game_id).await?))
}

async fn create_template(
  State(db): State<Database>, Json(body): Json<CreateTemplate>,
) -> Result<Json<isw_range_template::Model>, ResponseError> {
  let model = isw_range_template::Model {
    id: 0,
    created_at: Utc::now(),
    game_id: body.game_id,
    name: body.name,
    brief: body.brief,
    topology: body.topology,
  };
  Ok(Json(isw_range_template::create(&db.conn, model).await?))
}

async fn delete_template(
  State(db): State<Database>, Path(id): Path<i64>,
) -> Result<StatusCode, ResponseError> {
  isw_range_template::delete(&db.conn, id).await?;
  Ok(StatusCode::NO_CONTENT)
}

// ---- range instances ------------------------------------------------------

#[derive(Deserialize)]
struct CreateRange {
  template_id: i64,
  host_id: i64,
  #[serde(default)]
  group_index: i32,
  name: String,
}

#[derive(Serialize)]
struct RangeDetail {
  range: isw_range::Model,
  vms: Vec<isw_vm::Model>,
  flags: Vec<isw_flag::Model>,
}

async fn create_range(
  State(db): State<Database>, Json(body): Json<CreateRange>,
) -> Result<Json<isw_range::Model>, ResponseError> {
  let model = isw_range::Model {
    id: 0,
    created_at: Utc::now(),
    template_id: body.template_id,
    host_id: body.host_id,
    group_index: body.group_index,
    name: body.name,
    status: "pending".to_owned(),
    armed_at: None,
    snapshot_name: None,
    last_error: None,
  };
  Ok(Json(isw_range::create(&db.conn, model).await?))
}

async fn get_range(
  State(db): State<Database>, Path(id): Path<i64>,
) -> Result<Json<RangeDetail>, ResponseError> {
  let range = isw_range::get(&db.conn, id)
    .await?
    .ok_or_else(|| ResponseError::NotFound(format!("isw range {id}")))?;
  let vms = isw_vm::list_by_range(&db.conn, id).await?;
  let flags = isw_flag::list_by_range(&db.conn, id).await?;
  Ok(Json(RangeDetail { range, vms, flags }))
}

async fn delete_range(
  State(db): State<Database>, Path(id): Path<i64>,
) -> Result<StatusCode, ResponseError> {
  isw_range::delete(&db.conn, id).await?;
  Ok(StatusCode::NO_CONTENT)
}

/// Arm a range: mint + inject a fresh flag per bound challenge into the guests.
async fn arm_range(
  State(state): State<GlobalState>, Path(id): Path<i64>,
) -> Result<Json<ArmReport>, ResponseError> {
  Ok(Json(isw::arm_range(&state, id).await?))
}

/// Snapshot a range's guests to a clean baseline (run once, before arming).
async fn snapshot_range(
  State(state): State<GlobalState>, Path(id): Path<i64>,
) -> Result<StatusCode, ResponseError> {
  isw::snapshot_range(&state, id).await?;
  Ok(StatusCode::NO_CONTENT)
}

/// Reset a range: revert guests to the clean snapshot and re-inject fresh flags.
async fn reset_range(
  State(state): State<GlobalState>, Path(id): Path<i64>,
) -> Result<Json<ArmReport>, ResponseError> {
  Ok(Json(isw::reset_range(&state, id).await?))
}

// ---- team -> range assignments --------------------------------------------

#[derive(Deserialize)]
struct CreateAssignment {
  game_id: i64,
  range_id: i64,
  team_id: i64,
}

async fn create_assignment(
  State(db): State<Database>, Json(body): Json<CreateAssignment>,
) -> Result<Json<isw_assignment::Model>, ResponseError> {
  let model = isw_assignment::Model {
    id: 0,
    created_at: Utc::now(),
    game_id: body.game_id,
    range_id: body.range_id,
    team_id: body.team_id,
  };
  Ok(Json(isw_assignment::create(&db.conn, model).await?))
}

async fn delete_assignment(
  State(db): State<Database>, Path(id): Path<i64>,
) -> Result<StatusCode, ResponseError> {
  isw_assignment::delete(&db.conn, id).await?;
  Ok(StatusCode::NO_CONTENT)
}

// ---- VPN ------------------------------------------------------------------

#[derive(Deserialize)]
struct ProvisionVpn {
  team_id: i64,
  address: String,
  subnet: String,
}

/// Admin: provision a WireGuard peer for a team on a range; returns the client config.
async fn provision_vpn(
  State(state): State<GlobalState>, Path(id): Path<i64>, Json(body): Json<ProvisionVpn>,
) -> Result<String, ResponseError> {
  isw::provision_vpn(&state, id, body.team_id, body.address, body.subnet).await
}

/// Player: download my team's WireGuard config for its assigned range.
async fn get_my_vpn(
  State(state): State<GlobalState>, Extension(game): Extension<game::Model>,
  team_ext: Extension<Option<team::Model>>, Extension(token): Extension<Token>,
) -> Result<String, ResponseError> {
  let team = extract_team!(game, team_ext, token);
  let team = team.ok_or_else(|| {
    ResponseError::PreconditionFailed("join a team to access your range".to_owned())
  })?;
  isw::team_vpn_config(&state, game.id, team.id)
    .await?
    .ok_or_else(|| ResponseError::NotFound("no VPN config for your range yet".to_owned()))
}
