use std::sync::{Arc, atomic::AtomicBool};

use axum::{
  Extension,
  extract::{Request, State},
  http::header::{self, WWW_AUTHENTICATE},
  middleware::Next,
  response::IntoResponse,
};
use base64::Engine;
use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use r2s_cache::Cache;
use r2s_config::auth;
use r2s_database::{
  challenge, config, game, team,
  user::{self, Permission, Permissions},
};
use r2s_migrator::Database;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

use crate::{traits::ResponseError, utility::password::verify_password};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Token {
  pub id: i64,
  pub account: String,
  pub nickname: String,
  pub permissions: Permissions,
  pub exp: i64,
}

#[derive(Clone, Debug)]
pub struct TokenTracker {
  pub token: Arc<Mutex<Token>>,
  pub renew_requested: Arc<AtomicBool>,
  pub original: Option<String>,
}

pub async fn decode_token(token: &str, key: &str) -> Token {
  let token_data = match decode::<Token>(
    token,
    &DecodingKey::from_secret(key.as_ref()),
    &Validation::new(Algorithm::HS256),
  ) {
    Ok(c) => c,
    Err(err) => {
      debug!("decode token failed: {err:?}, token is {token:?}");
      return Token::default();
    }
  };
  token_data.claims
}

async fn distribute_token(
  token: &Token, key: &str, expires_time: i64,
) -> Result<String, ResponseError> {
  let new_token = Token {
    exp: Utc::now().timestamp() + expires_time,
    ..token.clone()
  };
  encode(
    &Header::default(),
    &new_token,
    &EncodingKey::from_secret(key.as_ref()),
  )
  .map_err(|err| {
    ResponseError::InternalServerError(
      "failed to encode token, please contact server admin".to_owned(),
      format!("failed to encode token: {err:?}"),
    )
  })
}

async fn extract_bearer_token(
  auth_config: &auth::Config, cache: &Cache, token: &str,
) -> Result<(Token, TokenTracker), ResponseError> {
  let token = token.split_whitespace().nth(1).unwrap_or_default();
  let valid = cache.at("token").exists(token).await?;

  let token_obj = if valid {
    decode_token(token, &auth_config.signing_key).await
  } else {
    Token::default()
  };
  debug!("user requested with token {token:?}, {token_obj:?}");

  let last_time = token_obj.exp - Utc::now().timestamp();

  let token_tracker = TokenTracker {
    token: Arc::new(Mutex::new(token_obj.clone())),
    renew_requested: Arc::new(AtomicBool::new(
      last_time > 0 && last_time < auth_config.buffer_time,
    )),
    original: Some(token.to_owned()),
  };

  debug!("extracted token: {token:?}");

  Ok((token_obj, token_tracker))
}

async fn extract_basic_token(
  db: &Database, cache: &Cache, token: &str,
) -> Result<(Token, TokenTracker), ResponseError> {
  // NOTE: we should limit login attempts here to prevent brute force attacks
  // this auth method is not recommended, but client such as git or curl may use
  // it, which is hard to integrate CAPTCHA protections

  // NOTE: the basic auth token is equivalent to a bearer token, include account
  // operations there may exists security risk, waiting for further discussion
  // limit the attempts to 5 times

  let (account, password) = {
    let decoded = base64::engine::general_purpose::STANDARD
      .decode(token.split_whitespace().nth(1).unwrap_or_default())
      .map_err(|_| ResponseError::BadRequest("Invalid basic token".to_owned()))?;
    let result = String::from_utf8(decoded)
      .map_err(|_| ResponseError::BadRequest("Invalid basic token".to_owned()))?;
    match result.split_once(':') {
      Some((account, password)) => (account.to_owned(), password.to_owned()),
      None => {
        return Err(ResponseError::BadRequest("Invalid basic token".to_owned()));
      }
    }
  };

  // debug!("user auth requested with basic auth: {account:?}, {password:?}");

  let attempts = cache.at("login").get::<i64>(&account).await?;
  if attempts.is_some_and(|attempts| attempts > 5) {
    return Err(ResponseError::TooManyRequests(
      "this account is frozen in 30 mins".to_owned(),
      format!("account {account} has too many login attempts"),
    ));
  }
  cache.at("login").incr(&account).await?;
  cache.at("login").expire(&account, 60 * 30).await?;

  let user = user::get_by_account_or_email(&db.conn, &account).await?;
  let user = match user {
    Some(user) => user,
    None => {
      return Err(ResponseError::Forbidden(
        "account or password is wrong".to_owned(),
        format!("user requested account {account} does not exist"),
      ));
    }
  };

  if user.banned || !user.permissions.0.contains(&Permission::Basic) {
    return Err(ResponseError::Forbidden(
      "account is banned".to_owned(),
      format!(
        "user {}:{} ({}) is banned",
        user.id, user.account, user.nickname
      ),
    ));
  }

  let password_hash = user.password.unwrap_or(String::new());

  match verify_password(&password, &password_hash)? {
    true => {
      info!(
        "User logged in with basic auth (oneshot): {}:{} ({}) <{}>",
        user.id,
        user.account,
        user.nickname,
        user.email.unwrap_or_default()
      );

      let token = Token {
        id: user.id,
        account: user.account,
        nickname: user.nickname,
        permissions: user.permissions,
        // NOTE: expires immediately here, only oneshot
        exp: Utc::now().timestamp(),
      };
      let token_tracker = TokenTracker {
        token: Arc::new(Mutex::new(token.clone())),
        renew_requested: Arc::new(AtomicBool::new(false)),
        original: None,
      };
      Ok((token, token_tracker))
    }
    false => Err(ResponseError::Forbidden(
      "account or password is wrong".to_owned(),
      format!(
        "user {}:{} ({}) requested with wrong password",
        user.id, user.account, user.nickname
      ),
    )),
  }
}

pub async fn extract_user_info(
  State(ref database): State<Database>, State(ref mut cache): State<Cache>,
  Extension(config): Extension<config::Model>, mut req: Request, next: Next,
) -> Result<impl IntoResponse, ResponseError> {
  let auth_config = config
    .auth
    .clone()
    .ok_or(ResponseError::InternalServerError(
      "auth config missing".into(),
      "auth section is not configured.".into(),
    ))?;
  debug!("auth req: {req:?}");
  let auth_header = req
    .headers()
    .get(header::AUTHORIZATION)
    .and_then(|header| header.to_str().ok());

  let token_str = if let Some(auth_header) = auth_header {
    auth_header.trim()
  } else {
    ""
  };

  let method = token_str.split_whitespace().next();
  let (token, token_tracker) = match method {
    Some("Bearer") => extract_bearer_token(&auth_config, cache, token_str).await?,
    Some("Basic") => extract_basic_token(database, cache, token_str).await?,
    Some(_) => {
      return Err(ResponseError::Unauthorized(
        "Unsupported auth method".to_owned(),
      ));
    }
    None => (
      Token::default(),
      TokenTracker {
        token: Arc::new(Mutex::new(Token::default())),
        renew_requested: Arc::new(AtomicBool::new(false)),
        original: None,
      },
    ),
  };

  req.extensions_mut().insert(token_tracker.clone());
  req.extensions_mut().insert(token.clone());

  let mut resp = next.run(req).await;

  if token_tracker
    .renew_requested
    .load(std::sync::atomic::Ordering::Relaxed)
  {
    let token_stored = token_tracker.token.lock().await.clone();
    match token_tracker.original.as_ref() {
      Some(t) => {
        cache.at("token").del(t).await.ok();
        cache
          .at("token")
          .rem(format!("user-{}", token_stored.id), t)
          .await
          .ok()
      }
      None => None,
    };
    let token_str = distribute_token(
      &token_stored,
      &auth_config.signing_key,
      auth_config.expires_time,
    )
    .await?;
    cache
      .at("token")
      .set_ex(&token_str, token_stored.id, auth_config.expires_time)
      .await
      .map_err(|err| {
        error!("failed to store new token: {:?}", err);
      })
      .ok();
    cache
      .at("token")
      .push(format!("user-{}", token_stored.id), &token_str)
      .await
      .ok();
    resp.headers_mut().insert(
      "Set-Token",
      token_str.parse().expect("failed to parse token"),
    );
  }

  Ok(resp)
}

pub async fn create_auth_header_by_user_agent(
  req: Request, next: Next,
) -> Result<impl IntoResponse, ResponseError> {
  let req_headers = req.headers();
  let user_agent = req_headers
    .get(header::USER_AGENT)
    .and_then(|header| header.to_str().ok())
    .unwrap_or_default()
    .to_owned();

  let mut resp = next.run(req).await;
  if resp.status() == 401 {
    let resp_headers = resp.headers_mut();
    // test git, docker and curl here
    match user_agent {
      u if u.contains("git") => {
        resp_headers.insert(
          WWW_AUTHENTICATE,
          header::HeaderValue::from_static("Basic realm=\"ret2shell\""),
        );
      }
      u if u.contains("docker") || u.contains("containers") => {
        resp_headers.insert(
          WWW_AUTHENTICATE,
          header::HeaderValue::from_static("Basic realm=\"ret2shell\""),
        );
      }
      u if u.contains("curl") => {
        resp_headers.insert(
          WWW_AUTHENTICATE,
          header::HeaderValue::from_static("Bearer realm=\"ret2shell\""),
        );
        resp_headers.insert(
          WWW_AUTHENTICATE,
          header::HeaderValue::from_static("Basic realm=\"ret2shell\""),
        );
      }
      _ => {}
    }
  }

  Ok(resp)
}

macro_rules! captcha_protected {
  ($cache:expr, $id:expr, $answer:expr) => {
    let captcha_id = $id;
    let captcha_answer = $answer;
    let captcha = $cache
      .at("captcha")
      .getdel::<r2s_captcha::Captcha>(captcha_id)
      .await?;
    if captcha.is_none() {
      return Err(ResponseError::Gone("captcha is outdate".to_owned()));
    }
    let captcha = captcha.unwrap();
    if !r2s_captcha::check(&captcha.validator, &captcha, captcha_answer).await? {
      return Err(ResponseError::Forbidden(
        "hey robot".to_owned(),
        "".to_owned(),
      ));
    }
  };
}

pub(crate) use captcha_protected;

#[allow(unused_macros)]
/// Construct a middleware closure that validate permissions from token.
///
/// all the permissions appeared here should be in the `permissions` field in
/// user's token.
///
/// Usage:
///
/// ```
/// Router::new()
///     .route(...)
///     .route_layer(axum::middleware::from_fn(permission_required_all!(Permission::Basic, ...)))
/// ```
macro_rules! permission_required_all {
    ($($perm:expr),*) => {
        |
            axum::extract::Extension(token): axum::extract::Extension<crate::middleware::auth::Token>,
            req: axum::extract::Request,
            next: axum::middleware::Next,
        | async move {
            if token.id <= 0 {
                return Err(crate::traits::ResponseError::Unauthorized("please login first".to_owned()));
            }
            let required_perms = [$($perm),*];
            tracing::debug!("user permissions: {:?}", token.permissions.0);
            tracing::debug!("required perms: {:?}", required_perms);
            match required_perms.iter().all(|perm| token.permissions.0.contains(perm)) {
                true => Ok(next.run(req).await),
                false => Err(crate::traits::ResponseError::Forbidden("permission denied".to_owned(), format!(
                    "user {}:{} ({}) want to access api '{}' without permission",
                    token.id, token.account, token.nickname, req.uri().path()
                )))
            }
        }
    };
}

#[allow(unused_macros)]
/// Construct a middleware closure that validate permissions from token.
///
/// all the permissions appeared here should be in the `permissions` field in
/// user's token.
///
/// Usage:
///
/// ```
/// Router::new()
///     .route(...)
///     .route_layer(axum::middleware::from_fn(permission_required_any!(Permission::Basic, ...)))
/// ```
macro_rules! permission_required_any {
    ($($perm:expr),*) => {
        |
            axum::extract::Extension(token): axum::extract::Extension<crate::middleware::auth::Token>,
            req: axum::extract::Request,
            next: axum::middleware::Next,
        | async move {
            if token.id <= 0 {
                return Err(crate::traits::ResponseError::Unauthorized("please login first".to_owned()));
            }
            let required_perms = [$($perm),*];
            tracing::debug!("user permissions: {:?}", token.permissions.0);
            tracing::debug!("required perms: {:?}", required_perms);
            match required_perms.iter().any(|perm| token.permissions.0.contains(perm)) {
                true => Ok(next.run(req).await),
                false => Err(crate::traits::ResponseError::Forbidden("permission denied".to_owned(), format!(
                    "user {}:{} ({}) want to access api '{}' without permission",
                    token.id, token.account, token.nickname, req.uri().path()
                )))
            }
        }
    };
}

#[allow(unused_imports)]
pub(crate) use permission_required_all;
#[allow(unused_imports)]
pub(crate) use permission_required_any;

use super::data::extract_team;

macro_rules! is_game_admin {
  ($token:expr, $game:expr) => {{ $token.permissions.0.contains(&Permission::Game) && $game.admins.0.contains(&$token.id) }};
}

pub(crate) use is_game_admin;

pub async fn game_admin_required(
  Extension(token): Extension<Token>, Extension(game): Extension<game::Model>, req: Request,
  next: Next,
) -> Result<impl IntoResponse, ResponseError> {
  if token.id <= 0 {
    return Err(ResponseError::Unauthorized("please login first".to_owned()));
  }
  if is_game_admin!(token, game) {
    Ok(next.run(req).await)
  } else {
    Err(ResponseError::Forbidden(
      "permission denied".to_owned(),
      format!(
        "user {}:{} ({}) want to access game {}:{} admin api with out permission",
        token.id, token.account, token.nickname, game.id, game.name
      ),
    ))
  }
}

pub async fn game_access_required(
  Extension(token): Extension<Token>, Extension(game): Extension<game::Model>,
  team_ext: Extension<Option<team::Model>>, req: Request, next: Next,
) -> Result<impl IntoResponse, ResponseError> {
  if token.id <= 0 {
    return Err(ResponseError::Unauthorized("please login first".to_owned()));
  }
  if is_game_admin!(token, game) {
    return Ok(next.run(req).await);
  }
  if game.hidden {
    return Err(ResponseError::Forbidden(
      "permission denied".to_owned(),
      format!(
        "user {}:{} ({}) want to access hidden game {}:{}",
        token.id, token.account, token.nickname, game.id, game.name
      ),
    ));
  }
  if game.host_type == game::HostType::Training {
    return Ok(next.run(req).await);
  }
  if game.archive_at < Utc::now() {
    return Ok(next.run(req).await);
  }

  let team = if let Extension(Some(team)) = team_ext {
    Some(team)
  } else {
    None
  };

  if team.is_none() || team.is_some_and(|team| team.state == team::State::Banned) {
    return Err(ResponseError::Forbidden(
      "permission denied".to_owned(),
      format!(
        "user {}:{} ({}) want to access game {}:{} api with out participation or banned",
        token.id, token.account, token.nickname, game.id, game.name
      ),
    ));
  }
  Ok(next.run(req).await)
}

pub async fn challenge_access_required(
  Extension(token): Extension<Token>, Extension(game): Extension<game::Model>,
  Extension(challenge): Extension<challenge::Model>, team_ext: Extension<Option<team::Model>>,
  req: Request, next: Next,
) -> Result<impl IntoResponse, ResponseError> {
  if token.id <= 0 {
    return Err(ResponseError::Unauthorized("please login first".to_owned()));
  }
  if game.id != challenge.game_id {
    return Err(ResponseError::Forbidden(
      "permission denied".to_owned(),
      format!(
        "user {}:{} ({}) want to access cross-game challenge {}:{} in game {}:{}",
        token.id, token.account, token.nickname, challenge.id, challenge.name, game.id, game.name
      ),
    ));
  }
  if is_game_admin!(token, game) {
    return Ok(next.run(req).await);
  }
  if game.hidden
    || challenge.hidden
    || game.frozen
    || challenge.release_at.is_some_and(|c| c > Utc::now())
  {
    return Err(ResponseError::Forbidden(
      "permission denied".to_owned(),
      format!(
        "user {}:{} ({}) want to access game {}:{} challenge {}:{} which is hidden or not released",
        token.id, token.account, token.nickname, game.id, game.name, challenge.id, challenge.name
      ),
    ));
  }
  if game.start_at > Utc::now() {
    return Err(ResponseError::PreconditionFailed(
      "game has not started".to_owned(),
    ));
  }
  let _team = extract_team!(game, team_ext, token);
  Ok(next.run(req).await)
}
