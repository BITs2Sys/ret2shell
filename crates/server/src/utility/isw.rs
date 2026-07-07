//! BITs2CTF fork: ISW range-mode flag orchestration + lifecycle.
//!
//! Attack-only flag hunt with **per-range shared flags**: mint one random flag per
//! (range, isw.toml challenge), inject it into the target guest via the host-agent,
//! and store only its sha256. Every team assigned to that range hunts the same
//! flag; submissions are verified by hash lookup (no rune checker), so the value is
//! distinct across range groups and can't be shared cross-group.
//!
//! Lifecycle: `snapshot_range` (baseline) → `arm_range` (power on + inject) →
//! `reset_range` (revert snapshot + power on + re-inject, since revert restores the
//! pre-injection disk).

use base64::{Engine, engine::general_purpose::STANDARD};
use nanoid::nanoid;
use r2s_bucket::{Bucket, challenge::ChallengeBucket};
use r2s_database::{
  challenge, game, isw_assignment, isw_flag, isw_host, isw_range, isw_range_template, isw_vm,
  isw_vpn_peer,
};
use r2s_isw::{
  AgentClient,
  protocol::{InjectRequest, PowerOp, VpnPeerRequest, VpnPeerResponse},
};
use serde::Serialize;
use tracing::{info, warn};

use crate::traits::{GlobalState, ResponseError};

const FLAG_ALPHABET: [char; 36] = [
  '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
  'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

const DEFAULT_SNAPSHOT: &str = "clean-armed";

fn sha256_hex(bytes: &[u8]) -> String {
  ring::digest::digest(&ring::digest::SHA256, bytes)
    .as_ref()
    .iter()
    .fold(String::new(), |mut acc, b| {
      acc.push_str(&format!("{b:02x}"));
      acc
    })
}

/// A freshly minted per-range flag.
fn mint_flag() -> String {
  format!("flag{{{}}}", nanoid!(32, &FLAG_ALPHABET))
}

#[derive(Debug, Serialize)]
pub struct FlagArmResult {
  pub challenge_id: i64,
  pub vm: String,
  pub guest_path: String,
  pub ok: bool,
  pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ArmReport {
  pub range_id: i64,
  pub flags: Vec<FlagArmResult>,
}

/// Everything needed to talk to a range's guests.
struct RangeContext {
  range: isw_range::Model,
  template: isw_range_template::Model,
  game: game::Model,
  client: AgentClient,
}

async fn get_challenge_bucket(
  bucket: &Bucket, game: &game::Model, challenge: &challenge::Model,
) -> Result<ChallengeBucket, ResponseError> {
  let game_bucket = game
    .bucket
    .clone()
    .ok_or_else(|| ResponseError::PreconditionFailed(format!("game {} has no bucket", game.id)))?;
  let challenge_bucket = challenge.bucket.clone().ok_or_else(|| {
    ResponseError::PreconditionFailed(format!("challenge {} has no bucket", challenge.id))
  })?;
  Ok(bucket.at(game_bucket).await?.at(challenge_bucket).await?)
}

async fn load_context(state: &GlobalState, range_id: i64) -> Result<RangeContext, ResponseError> {
  let db = &state.db.conn;
  let range = isw_range::get(db, range_id)
    .await?
    .ok_or_else(|| ResponseError::NotFound(format!("isw range {range_id}")))?;
  let template = isw_range_template::get(db, range.template_id)
    .await?
    .ok_or_else(|| ResponseError::NotFound("isw range template".to_owned()))?;
  let host = isw_host::get(db, range.host_id)
    .await?
    .ok_or_else(|| ResponseError::NotFound("isw host".to_owned()))?;
  let game = game::get(db, template.game_id)
    .await?
    .ok_or_else(|| ResponseError::NotFound("game".to_owned()))?;
  let client = state.isw.client_for(&host.address, host.api_port);
  Ok(RangeContext {
    range,
    template,
    game,
    client,
  })
}

/// Ensure `isw_vm` rows exist for the template topology.
async fn sync_vm_rows(state: &GlobalState, ctx: &RangeContext) -> Result<(), ResponseError> {
  let db = &state.db.conn;
  for spec in &ctx.template.topology.vms {
    if isw_vm::get_by_logical(db, ctx.range.id, &spec.logical_name)
      .await?
      .is_none()
    {
      isw_vm::create(
        db,
        isw_vm::Model {
          id: 0,
          created_at: chrono::Utc::now(),
          range_id: ctx.range.id,
          logical_name: spec.logical_name.clone(),
          guest_os: spec.guest_os.clone(),
          vmx_path: spec.vmx.clone(),
          ip: None,
          power_state: "unknown".to_owned(),
          tools_state: "unknown".to_owned(),
        },
      )
      .await?;
    }
  }
  Ok(())
}

/// Mint + inject a fresh flag for every enabled `isw.toml` challenge bound to this
/// range's template, verifying each by host-side read-back hash.
async fn mint_and_inject(
  state: &GlobalState, ctx: &RangeContext,
) -> Result<Vec<FlagArmResult>, ResponseError> {
  let db = &state.db.conn;
  isw_flag::clear_range(db, ctx.range.id).await?;
  let challenges = challenge::get_full_list(db, ctx.template.game_id).await?;
  let mut results = Vec::new();
  for challenge in challenges {
    let Ok(challenge_bucket) = get_challenge_bucket(&state.bucket, &ctx.game, &challenge).await
    else {
      continue;
    };
    let Some(cfg) = challenge_bucket.isw().await.ok().flatten() else {
      continue;
    };
    if !cfg.enabled || cfg.range_template != ctx.template.name {
      continue;
    }

    let flag = mint_flag();
    let hash = sha256_hex(flag.as_bytes());
    let vm_row = isw_vm::get_by_logical(db, ctx.range.id, &cfg.vm).await?;
    let record = isw_flag::create(
      db,
      isw_flag::Model {
        id: 0,
        created_at: chrono::Utc::now(),
        range_id: ctx.range.id,
        challenge_id: challenge.id,
        vm_id: vm_row.as_ref().map(|vm| vm.id),
        guest_path: cfg.guest_path.clone(),
        value_hash: hash.clone(),
        round: 0,
        injected_at: None,
        verified: false,
        last_error: None,
      },
    )
    .await?;

    let inject = ctx
      .client
      .inject(
        &cfg.vm,
        &InjectRequest {
          guest_path: cfg.guest_path.clone(),
          content_b64: STANDARD.encode(flag.as_bytes()),
          owner: cfg.owner.clone(),
          mode: Some(cfg.mode.clone()),
        },
      )
      .await;

    let (ok, message) = match inject {
      Ok(result) if result.ok && result.sha256.as_deref() == Some(hash.as_str()) => {
        isw_flag::mark_verified(db, record.id, true).await?;
        (true, None)
      }
      Ok(result) => {
        let msg = result
          .message
          .unwrap_or_else(|| "inject/verify mismatch".to_owned());
        isw_flag::mark_error(db, record.id, &msg).await?;
        (false, Some(msg))
      }
      Err(err) => {
        let msg = err.to_string();
        isw_flag::mark_error(db, record.id, &msg).await?;
        (false, Some(msg))
      }
    };
    results.push(FlagArmResult {
      challenge_id: challenge.id,
      vm: cfg.vm.clone(),
      guest_path: cfg.guest_path.clone(),
      ok,
      message,
    });
  }
  Ok(results)
}

async fn finalize_status(
  state: &GlobalState, ctx: &RangeContext, results: &[FlagArmResult],
) -> Result<(), ResponseError> {
  let armed_ok = !results.is_empty() && results.iter().all(|r| r.ok);
  // Distinguish "no isw.toml challenge targeted this template" (empty results) from a
  // genuine inject failure, so the error state is actionable for admins.
  let last_error = if results.is_empty() {
    Some(format!(
      "no enabled isw.toml challenge targets range template `{}`",
      ctx.template.name
    ))
  } else if !armed_ok {
    results.iter().find_map(|r| r.message.clone())
  } else {
    None
  };
  isw_range::update_state(
    &state.db.conn,
    isw_range::Model {
      id: ctx.range.id,
      created_at: ctx.range.created_at,
      template_id: ctx.range.template_id,
      host_id: ctx.range.host_id,
      group_index: ctx.range.group_index,
      name: ctx.range.name.clone(),
      status: if armed_ok { "armed" } else { "error" }.to_owned(),
      armed_at: Some(chrono::Utc::now()),
      snapshot_name: ctx.range.snapshot_name.clone(),
      last_error,
    },
  )
  .await?;
  Ok(())
}

/// Arm a range: sync VM rows, best-effort power on, then mint + inject flags.
pub async fn arm_range(state: &GlobalState, range_id: i64) -> Result<ArmReport, ResponseError> {
  let ctx = load_context(state, range_id).await?;
  sync_vm_rows(state, &ctx).await?;
  for spec in &ctx.template.topology.vms {
    if let Err(err) = ctx.client.power(&spec.logical_name, PowerOp::Start).await {
      warn!(vm = %spec.logical_name, error = %err, "isw: failed to power on guest (continuing)");
    }
  }
  let results = mint_and_inject(state, &ctx).await?;
  finalize_status(state, &ctx, &results).await?;
  info!(range_id, flags = results.len(), "isw range armed");
  Ok(ArmReport {
    range_id,
    flags: results,
  })
}

/// Snapshot the range's guests to a clean baseline (run once, before arming).
pub async fn snapshot_range(state: &GlobalState, range_id: i64) -> Result<(), ResponseError> {
  let ctx = load_context(state, range_id).await?;
  sync_vm_rows(state, &ctx).await?;
  let name = ctx
    .range
    .snapshot_name
    .clone()
    .unwrap_or_else(|| DEFAULT_SNAPSHOT.to_owned());
  for spec in &ctx.template.topology.vms {
    if let Err(err) = ctx.client.snapshot(&spec.logical_name, &name).await {
      warn!(vm = %spec.logical_name, error = %err, "isw: snapshot failed");
    }
  }
  // persist the snapshot name so reset knows what to revert to.
  isw_range::update_state(
    &state.db.conn,
    isw_range::Model {
      snapshot_name: Some(name),
      ..ctx.range.clone()
    },
  )
  .await?;
  info!(range_id, "isw range snapshotted");
  Ok(())
}

/// Reset a range: revert each guest to its clean snapshot, power on, and re-inject
/// fresh flags (the revert restores the pre-injection disk, so flags must be re-run).
pub async fn reset_range(state: &GlobalState, range_id: i64) -> Result<ArmReport, ResponseError> {
  let ctx = load_context(state, range_id).await?;
  let name = ctx
    .range
    .snapshot_name
    .clone()
    .unwrap_or_else(|| DEFAULT_SNAPSHOT.to_owned());
  for spec in &ctx.template.topology.vms {
    if let Err(err) = ctx.client.revert(&spec.logical_name, &name).await {
      warn!(vm = %spec.logical_name, error = %err, "isw: revert failed (continuing)");
    }
    if let Err(err) = ctx.client.power(&spec.logical_name, PowerOp::Start).await {
      warn!(vm = %spec.logical_name, error = %err, "isw: power on after revert failed");
    }
  }
  let results = mint_and_inject(state, &ctx).await?;
  finalize_status(state, &ctx, &results).await?;
  info!(range_id, flags = results.len(), "isw range reset + re-armed");
  Ok(ArmReport {
    range_id,
    flags: results,
  })
}

fn build_wg_config(resp: &VpnPeerResponse, address: &str, subnet: &str) -> String {
  let dns = resp
    .dns
    .as_ref()
    .map(|d| format!("DNS = {d}\n"))
    .unwrap_or_default();
  format!(
    "[Interface]\nPrivateKey = {}\nAddress = {}/32\n{}\n[Peer]\nPublicKey = {}\nEndpoint = \
     {}\nAllowedIPs = {}\nPersistentKeepalive = 25\n",
    resp.client_private_key, address, dns, resp.server_public_key, resp.endpoint, subnet
  )
}

/// Provision a WireGuard peer for a team on its range and return the client config.
/// The full config (incl. the client private key) is stored in `isw_vpn_peer` so the
/// team can re-download it; the DB already holds the platform's other secrets.
pub async fn provision_vpn(
  state: &GlobalState, range_id: i64, team_id: i64, address: String, subnet: String,
) -> Result<String, ResponseError> {
  let ctx = load_context(state, range_id).await?;
  let resp = ctx
    .client
    .provision_vpn(&VpnPeerRequest {
      address: address.clone(),
      subnet: subnet.clone(),
    })
    .await
    .map_err(|e| ResponseError::InternalServerError(e.to_string()))?;
  let config = build_wg_config(&resp, &address, &subnet);
  // idempotency: revoke any prior peer(s) for this (range, team) so exactly one active
  // peer/config exists and get_for_team returns the one we just handed out (repeated
  // provisioning otherwise piles up duplicate rows with mismatched keys).
  for existing in isw_vpn_peer::list_by_range(&state.db.conn, range_id).await? {
    if existing.team_id == team_id && !existing.revoked {
      isw_vpn_peer::set_revoked(&state.db.conn, existing.id, true)
        .await
        .ok();
    }
  }
  isw_vpn_peer::create(
    &state.db.conn,
    isw_vpn_peer::Model {
      id: 0,
      created_at: chrono::Utc::now(),
      range_id,
      team_id,
      public_key: resp.client_public_key,
      address,
      config_ref: Some(config.clone()),
      revoked: false,
    },
  )
  .await?;
  info!(range_id, team_id, "isw vpn peer provisioned");
  Ok(config)
}

/// Fetch a team's stored WireGuard config for their assigned range (player download).
pub async fn team_vpn_config(
  state: &GlobalState, game_id: i64, team_id: i64,
) -> Result<Option<String>, ResponseError> {
  let db = &state.db.conn;
  let Some(assignment) = isw_assignment::get_for_team(db, game_id, team_id).await? else {
    return Ok(None);
  };
  let Some(peer) = isw_vpn_peer::get_for_team(db, assignment.range_id, team_id).await? else {
    return Ok(None);
  };
  if peer.revoked {
    return Ok(None);
  }
  Ok(peer.config_ref)
}

/// Verify a submitted flag for an ISW challenge: resolve the submitting team's
/// range, look up the injected flag for (range, challenge), and hash-compare.
/// Generic over the connection so it runs inside the submission worker's txn.
pub async fn verify_submission<C>(
  db: &C, game_id: i64, challenge_id: i64, team_id: Option<i64>, content: &str,
) -> Result<bool, ResponseError>
where
  C: sea_orm::ConnectionTrait, {
  let Some(team_id) = team_id else {
    return Ok(false);
  };
  let Some(assignment) = isw_assignment::get_for_team(db, game_id, team_id).await? else {
    return Ok(false);
  };
  let Some(flag) = isw_flag::get_current(db, assignment.range_id, challenge_id).await? else {
    return Ok(false);
  };
  // Only score against a flag confirmed present in the guest by read-back hash. An
  // unverified row (inject/revert failed) must not accept or reject real submissions —
  // otherwise the phantom hash makes the challenge unsolvable.
  if !flag.verified {
    return Ok(false);
  }
  Ok(sha256_hex(content.trim().as_bytes()) == flag.value_hash)
}
