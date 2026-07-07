//! BITs2CTF fork (A6): Fix challenge-kind worker, moved out of the core
//! `worker/game.rs` into this fork-owned module so upstream refactors of the game
//! workers don't collide with the fork. Behaviour is unchanged. The shared
//! `get_award_rate` stays in `game.rs` and is reused via `super::game`.
#![allow(clippy::too_many_arguments)]

/// Result string stamped on a fix submission whose tester never produced a verdict
/// (pod launch / checker infra failure). Such submissions must NOT consume an attempt
/// slot — the attempt-count query excludes them.
pub(crate) const FIX_CHECKER_INTERNAL_ERROR: &str = "fix checker internal error, incorrect.";

use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

use chrono::Utc;
use futures::StreamExt;
use r2s_bucket::Bucket;
use r2s_cache::Cache;
use r2s_checker::Checker;
use r2s_cluster::{CHALLENGE_NS, Cluster};
use r2s_config::GlobalConfig;
use r2s_database::{
  challenge, extra, game, submission,
  team::{self, TeamScoreHistory, TeamScoreHistoryKind},
  user,
};
use r2s_engine::Engine;
use r2s_event::{
  Event,
  events::{EventContainer, SubmissionEvent, SubmissionEventType},
};
use r2s_migrator::Database;
use r2s_queue::{Queue, TracedMessage};
use sea_orm::TransactionTrait;
use tokio::io::AsyncWriteExt;
use tracing::{Instrument, Span, debug, error, error_span, warn};

use super::game::get_award_rate;
use crate::{
  traits::ResponseError,
  utility::fix::{
    decode_fix_submission_meta, fix_upload_dir, fix_upload_path, parse_fix_result, shell_quote,
  },
};

pub(super) async fn fix_worker(
  queue: Queue, db: Database, cache: Cache, engine: Engine, checker: Checker, bucket: Bucket,
  cluster: Cluster, config: GlobalConfig,
) {
  let messages = queue
    .subscribe("fix")
    .await
    .inspect_err(|err| {
      error!(error=?err, "failed to subscribe to fix-check queue");
    })
    .ok();
  let mut messages = if let Some(messages) = messages {
    messages
  } else {
    return;
  };

  // BITs2CTF fork (B3): on startup no fix check is legitimately running, so sweep
  // any crash-orphaned fix pods first. Their names are keyed by submission id, so
  // a stale pod would otherwise block a redelivered submission from re-running.
  reconcile_fix_pods(&cluster).await;

  while let Some(message) = messages.next().await {
    if let Ok(message) = message {
      let req = String::from_utf8(message.message.payload.to_vec())
        .inspect_err(|e| {
          error!(error=?e, "failed to parse message from nats");
        })
        .ok();
      if req.is_none() {
        message.double_ack().await.ok();
        continue;
      }
      let submission_msg = serde_json::from_str::<TracedMessage<submission::Model>>(&req.unwrap())
        .inspect_err(|e| {
          error!(error=?e, "failed to parse message from nats");
        })
        .ok();
      if submission_msg.is_none() {
        message.double_ack().await.ok();
        continue;
      }
      let submission_msg = submission_msg.unwrap();
      let trace = submission_msg.trace.to_owned();
      let Some(Some(submission)) = submission::get(&db.conn, submission_msg.payload.id)
        .await
        .inspect_err(|e| error!(error=?e, "failed to load fix submission from database"))
        .ok()
      else {
        message.double_ack().await.ok();
        continue;
      };
      if submission.result.is_some() || submission.solved.is_some() {
        debug!(
          submission_id = submission.id,
          "fix submission already processed, skip message"
        );
        message.double_ack().await.ok();
        continue;
      }
      let span = error_span!(
        "request", trace=%trace,
        "data-submission-id"=%submission.id,
        "data-submission-content"=?submission.content,
        "user-id"=tracing::field::Empty,
        "user-account"=tracing::field::Empty,
        "user-nickname"=tracing::field::Empty,
        "team-id"=tracing::field::Empty,
        "team-name"=tracing::field::Empty,
        "data-challenge-id"=%submission.challenge_id,
        "data-challenge-name"=tracing::field::Empty,
        "data-game-id"=tracing::field::Empty,
        "data-game-name"=tracing::field::Empty
      );
      let result = fix_worker_exec(
        queue.clone(),
        db.clone(),
        cache.clone(),
        engine.clone(),
        checker.clone(),
        bucket.clone(),
        cluster.clone(),
        config.clone(),
        &submission,
        &trace,
      )
      .instrument(span)
      .await
      .inspect_err(|e| error!(error=?e, "failed to process fix submission"))
      .ok();
      if result.is_none() {
        submission::update(
          &db.conn,
          submission::Model {
            id: submission.id,
            solved: Some(false),
            result: Some(FIX_CHECKER_INTERNAL_ERROR.to_owned()),
            ..submission
          },
        )
        .await
        .ok();
      }
      message.double_ack().await.ok();
    } else {
      error!(error=?message, "failed to receive message from nats");
    }
  }
}

/// BITs2CTF fork (B3): delete crash-orphaned fix pods (and any services) so a
/// previous worker crash can't leak instances or block redelivered submissions.
async fn reconcile_fix_pods(cluster: &Cluster) {
  let cluster = cluster.at(CHALLENGE_NS);
  match cluster.get_pods_by_label("ret.sh.cn/fix=true").await {
    Ok(pods) => {
      for pod in pods {
        if let Some(name) = pod.metadata.name {
          cluster.delete_service(&name).await.ok();
          cluster.delete_pod(&name).await.ok();
        }
      }
    }
    Err(err) => warn!(error = ?err, "failed to reconcile orphaned fix pods on startup"),
  }
}

async fn fix_worker_exec(
  queue: Queue, db: Database, cache: Cache, engine: Engine, checker: Checker, bucket: Bucket,
  cluster: Cluster, config: GlobalConfig, submission: &submission::Model, trace: impl AsRef<str>,
) -> Result<submission::Model, ResponseError> {
  let meta = decode_fix_submission_meta(submission.content.as_deref()).ok_or(
    ResponseError::BadRequest("invalid fix submission metadata".to_owned()),
  )?;
  let challenge = challenge::get(&db.conn, submission.challenge_id)
    .await?
    .ok_or(ResponseError::BadRequest("challenge not found".to_owned()))?;
  Span::current().record("data-challenge-name", challenge.name.as_str());
  Span::current().record("data-game-id", challenge.game_id);
  let game = game::get(&db.conn, challenge.game_id)
    .await?
    .ok_or(ResponseError::BadRequest("game not found".to_owned()))?;
  Span::current().record("data-game-name", game.name.as_str());
  let user = user::get(&db.conn, submission.user_id)
    .await?
    .ok_or(ResponseError::BadRequest("user not found".to_owned()))?;
  Span::current().record("user-id", user.id);
  Span::current().record("user-account", user.account.as_str());
  Span::current().record("user-nickname", user.nickname.as_str());
  let team = if let Some(team_id) = submission.team_id {
    team::get(&db.conn, team_id).await?
  } else {
    None
  };
  if let Some(team) = &team {
    Span::current().record("team-id", team.id);
    Span::current().record("team-name", team.name.as_str());
  }
  let prev_submitted = submission::count(
    &db.conn,
    true,
    Some(challenge.game_id),
    Some(challenge.id),
    submission.team_id,
    None,
    None,
    false,
  )
  .await?
    > 0;
  let challenge_bucket = bucket
    .at(
      game
        .bucket
        .clone()
        .ok_or(ResponseError::PreconditionFailed(format!(
          "game {}:{} does not have a valid bucket",
          game.id, game.name
        )))?,
    )
    .await?
    .at(
      challenge
        .bucket
        .clone()
        .ok_or(ResponseError::PreconditionFailed(format!(
          "challenge {}:{} in game {}:{} does not have a valid bucket",
          challenge.id, challenge.name, game.id, game.name
        )))?,
    )
    .await?;
  let fix_config = challenge_bucket
    .fix()
    .await?
    .ok_or(ResponseError::PreconditionFailed(
      "this challenge is not a fix challenge".to_owned(),
    ))?;
  if !fix_config.enabled {
    return Err(ResponseError::PreconditionFailed(
      "this challenge is not a fix challenge".to_owned(),
    ));
  }

  let (solved, result) = run_fix_check(
    &cluster,
    &config,
    &engine,
    &checker,
    &challenge_bucket,
    &game,
    &challenge,
    &user,
    &team,
    submission,
    &fix_config,
    &meta,
  )
  .await
  .unwrap_or_else(|err| {
    warn!(error=?err, "fix check failed");
    (false, format!("fix check failed: {err}"))
  });

  let txn = db.conn.begin().await?;
  let submission = submission::update(
    &txn,
    submission::Model {
      id: submission.id,
      solved: Some(solved),
      result: Some(result),
      ..submission.clone()
    },
  )
  .await?;

  if !solved || team.is_none() || prev_submitted {
    txn.commit().await?;
    return Ok(submission);
  }

  let mut team = team.unwrap();
  let (changed, decay, challenge) = challenge::maintain_score(&txn, challenge.clone()).await?;
  let blood_state = if decay <= 3 { Some(decay as i32) } else { None };
  let changed_at = submission.created_at;
  if let Some(blood_state) = blood_state {
    let score = challenge.score_rule.initial * get_award_rate(&game, blood_state) / 100;
    if score > 0 {
      extra::create(
        &txn,
        extra::Model {
          id: 0,
          created_at: changed_at,
          team_id: team.id,
          challenge_id: Some(challenge.id),
          score,
          reason: format!(
            "No.{blood_state} fix for challenge {}:{}",
            challenge.id, challenge.name
          ),
          hint_id: None,
        },
      )
      .await?;
    }
  }
  let score = team::calc_score(&txn, team.id).await?;
  team.score = score;
  team.history.0.push(TeamScoreHistory {
    changed_at,
    blood_state,
    challenge_id: Some(challenge.id),
    kind: TeamScoreHistoryKind::Solve,
    score,
  });
  team.last_active_at = changed_at;
  team::update(&txn, team.clone()).await?;

  let event = EventContainer {
    game_id: challenge.game_id,
    event: Event::Submission(Box::new(SubmissionEvent {
      event_type: SubmissionEventType::Correct,
      submission: submission.clone(),
      blood_state,
      challenge: challenge.clone(),
      operator: user.clone(),
      team: Some(team),
      peer_team: None,
      reason: None,
    })),
  };
  txn.commit().await?;
  queue.publish("event", event, &trace).await.ok();
  if changed {
    queue
      .publish("scoreboard", challenge.clone(), &trace)
      .await
      .ok();
    cache.at("challenge").del(challenge.id).await.ok();
  }

  Ok(submission)
}

#[allow(clippy::too_many_arguments)]
async fn run_fix_check(
  cluster: &Cluster, config: &GlobalConfig, engine: &Engine, checker: &Checker,
  challenge_bucket: &r2s_bucket::challenge::ChallengeBucket, game: &game::Model,
  challenge: &challenge::Model, user: &user::Model, team: &Option<team::Model>,
  submission: &submission::Model, fix_config: &r2s_config::cluster::FixConfig,
  meta: &crate::utility::fix::FixSubmissionMeta,
) -> Result<(bool, String), ResponseError> {
  let env_config = challenge_bucket
    .env()
    .await?
    .ok_or(ResponseError::PreconditionFailed(
      "fix challenge requires an online environment".to_owned(),
    ))?;
  if env_config.images.is_empty() || env_config.images.iter().all(|image| image.port.is_none()) {
    return Err(ResponseError::PreconditionFailed(
      "fix challenge target requires at least one service port".to_owned(),
    ));
  }
  let tester = fix_config
    .tester
    .clone()
    .ok_or(ResponseError::PreconditionFailed(
      "fix tester image is required".to_owned(),
    ))?;
  let target_port = fix_config
    .target_port
    .or_else(|| env_config.images.iter().find_map(|image| image.port))
    .ok_or(ResponseError::PreconditionFailed(
      "fix challenge target port is required".to_owned(),
    ))?;

  let mut env_map = match checker.preload(engine, challenge, challenge_bucket).await {
    Ok(_) => checker
      .environ(engine, challenge_bucket, user, team)
      .await
      .unwrap_or_default(),
    Err(err) => {
      warn!(error=?err, "failed to preload checker for fix environment, using empty env");
      HashMap::new()
    }
  };
  env_map.insert(
    "R2S_FIX_SUBMISSION_ID".to_owned(),
    submission.id.to_string(),
  );
  env_map.insert("R2S_FIX_ORIGINAL_NAME".to_owned(), meta.file_name.clone());

  let node_selector = if game.archive_at > Utc::now() {
    game.node_selector.clone().or_else(|| {
      config
        .cluster
        .as_ref()
        .and_then(|config| config.node_selector.clone())
    })
  } else {
    config
      .cluster
      .as_ref()
      .and_then(|config| config.node_selector.clone())
  }
  .and_then(|node_selector| {
    if node_selector.is_empty() {
      None
    } else {
      Some(node_selector)
    }
  });

  let traffic = format!("fix-{}", submission.id);
  let target_labels = [
    ("ret.sh.cn/fix", "true".to_owned()),
    ("ret.sh.cn/fix-role", "target".to_owned()),
    ("ret.sh.cn/fix-submission", submission.id.to_string()),
    ("ret.sh.cn/fix-challenge", challenge.id.to_string()),
    (
      "ret.sh.cn/fix-team",
      submission.team_id.unwrap_or_default().to_string(),
    ),
    ("ret.sh.cn/fix-user", user.id.to_string()),
    ("ret.sh.cn/fix-traffic", traffic),
    ("ret.sh.cn/internet", env_config.internet.to_string()),
  ]
  .iter()
  .map(|(k, v)| (k.to_string(), v.to_owned()))
  .collect::<BTreeMap<_, _>>();
  let target_annotations = [
    ("ret.sh.cn/challenge", challenge.name.clone()),
    ("ret.sh.cn/game", game.name.clone()),
    ("ret.sh.cn/user", user.account.clone()),
    ("ret.sh.cn/user-nickname", user.nickname.clone()),
    ("ret.sh.cn/ports", target_port.to_string()),
  ]
  .iter()
  .map(|(k, v)| (k.to_string(), v.to_owned()))
  .collect::<BTreeMap<_, _>>();

  let cluster = cluster.at(CHALLENGE_NS);
  let timeout = Duration::from_secs(fix_config.timeout_secs);
  let snapshot = cluster
    .create_fix_target_env(
      submission.id,
      target_labels,
      target_annotations,
      env_map,
      env_config,
      node_selector.clone(),
    )
    .await?;
  let target_name = snapshot
    .pod
    .metadata
    .name
    .clone()
    .ok_or(ResponseError::PreconditionFailed(
      "fix target pod has no name".to_owned(),
    ))?;
  let target_container = fix_config.target_container.as_deref();
  let check_result = async {
    cluster.wait_pod_running(&target_name, timeout).await?;
    let upload_path = fix_upload_path(&meta.token);
    let upload = cluster
      .upload_file_to_pod(
        &target_name,
        target_container,
        &upload_path,
        &fix_config.upload_path,
      )
      .await?;
    if !upload.success {
      return Ok::<_, ResponseError>((
        false,
        format!("failed to upload fix artifact: {}", upload.stderr),
      ));
    }

    let script_tmp = fix_upload_dir(&meta.token).join("fix.sh");
    let mut script_src = challenge_bucket
      .download_checker(&fix_config.fix_script)
      .await?;
    let mut script_dst = tokio::fs::File::create(&script_tmp).await?;
    tokio::io::copy(&mut script_src, &mut script_dst).await?;
    script_dst.flush().await?;
    let script_path = "/tmp/ret2shell-fix/fix.sh";
    let upload_script = cluster
      .upload_file_to_pod(&target_name, target_container, &script_tmp, script_path)
      .await?;
    if !upload_script.success {
      return Ok((
        false,
        format!("failed to upload fix script: {}", upload_script.stderr),
      ));
    }
    let command = vec![
      "/bin/sh".to_owned(),
      "-c".to_owned(),
      format!(
        "chmod +x {} && R2S_FIX_UPLOAD={} R2S_FIX_ORIGINAL_NAME={} R2S_FIX_WORKDIR=/tmp/ret2shell-fix {}",
        shell_quote(script_path),
        shell_quote(&fix_config.upload_path),
        shell_quote(&meta.file_name),
        shell_quote(script_path)
      ),
    ];
    let fix_output = cluster
      .exec_pod(&target_name, target_container, command, None, timeout)
      .await?;
    if !fix_output.success {
      return Ok((
        false,
        format!(
          "fix script failed: {}{}",
          fix_output.stderr,
          fix_output
            .reason
            .as_ref()
            .map(|reason| format!(" ({reason})"))
            .unwrap_or_default()
        ),
      ));
    }

    let tester_name = format!("ret2shell-fix-test-{}", submission.id);
    let tester_labels = [
      ("ret.sh.cn/fix", "true".to_owned()),
      ("ret.sh.cn/fix-role", "tester".to_owned()),
      ("ret.sh.cn/fix-submission", submission.id.to_string()),
      ("ret.sh.cn/fix-challenge", challenge.id.to_string()),
      ("ret.sh.cn/fix-user", user.id.to_string()),
      ("ret.sh.cn/internet", "false".to_owned()),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_owned()))
    .collect::<BTreeMap<_, _>>();
    let tester_annotations = [
      ("ret.sh.cn/challenge", challenge.name.clone()),
      ("ret.sh.cn/game", game.name.clone()),
      ("ret.sh.cn/user", user.account.clone()),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_owned()))
    .collect::<BTreeMap<_, _>>();
    let mut tester_env = HashMap::new();
    tester_env.insert("R2S_FIX_TARGET_HOST".to_owned(), target_name.clone());
    tester_env.insert("R2S_FIX_TARGET_PORT".to_owned(), target_port.to_string());
    tester_env.insert(
      "R2S_FIX_TARGET_URL".to_owned(),
      format!("http://{target_name}:{target_port}"),
    );
    tester_env.insert("R2S_FIX_RESULT_ENV".to_owned(), fix_config.result_env.clone());
    tester_env.insert("R2S_FIX_RESULT".to_owned(), "failed".to_owned());
    cluster
      .create_fix_tester_pod(
        &tester_name,
        tester_labels,
        tester_annotations,
        tester_env,
        tester.clone(),
        fix_config.pull_secret.clone(),
        fix_config.tester_command.clone(),
        node_selector,
      )
      .await?;
    let tester_pod = cluster.wait_pod_finished(&tester_name, timeout).await;
    let logs = cluster
      .pod_logs_string(tester_name.clone(), Some(tester.name.clone()))
      .await
      .unwrap_or_default();
    cluster.delete_pod(&tester_name).await.ok();
    let tester_pod = tester_pod?;
    let tester_succeeded = tester_pod
      .status
      .as_ref()
      .and_then(|status| status.phase.as_deref())
      == Some("Succeeded");
    let result_value = parse_fix_result(&logs, &fix_config.result_env);
    if tester_succeeded && result_value.as_deref() == Some(fix_config.success_value.as_str()) {
      Ok((true, "fix accepted".to_owned()))
    } else {
      Ok((
        false,
        format!(
          "fix rejected: tester status={}, {}={}",
          tester_pod
            .status
            .and_then(|status| status.phase)
            .unwrap_or_else(|| "Unknown".to_owned()),
          fix_config.result_env,
          result_value.unwrap_or_else(|| "<missing>".to_owned())
        ),
      ))
    }
  }
  .await;
  cluster.delete_service(&target_name).await.ok();
  cluster.delete_pod(&target_name).await.ok();
  tokio::fs::remove_dir_all(fix_upload_dir(&meta.token))
    .await
    .ok();
  check_result
}
