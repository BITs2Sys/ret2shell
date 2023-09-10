//! Authentication & Authorization middlewares.
//!
//! permissions for platform controller:
//!
//! ```
//! ["basic", "verified", "publish", "audit", "organize", "devops", "statistics", "ctftime", "certificates"]
//! ```
//!

use axum::{
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Extension,
};
use chrono::Local;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::{atomic::AtomicBool, Arc};
use tokio::sync::Mutex;
use tracing::{debug, error, warn};

use crate::{
    cache::{self, manager::RedisPool},
    config::GlobalConfig,
    entity::{
        config::{Auth, Model as ConfigModel},
        user::{Permission, Permissions},
    },
};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Token {
    pub id: i64,
    pub name: String,
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
            return Token {
                id: -1,
                ..Default::default()
            };
        }
    };
    token_data.claims
}

async fn distribute_token(token: Token, key: &str, expires_time: i64) -> String {
    let new_token = Token {
        exp: Local::now().timestamp() + expires_time,
        ..token
    };
    let token = encode(
        &Header::default(),
        &new_token,
        &EncodingKey::from_secret(key.as_ref()),
    )
    .expect("Failed to encode token");
    token
}

pub async fn extract_user_info<B>(
    State(ref mut cache): State<RedisPool>,
    config: Option<Extension<ConfigModel>>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    if let Some(Extension(config)) = config {
        let auth_config = config.auth;
        let Auth {
            ref signing_key,
            buffer_time,
            expires_time,
        } = auth_config;
        let auth_header = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|header| header.to_str().ok());

        let auth_header = if let Some(auth_header) = auth_header {
            auth_header
                .strip_prefix("Bearer ")
                .unwrap_or(&auth_header)
                .trim()
        } else {
            ""
        };

        let auth_header = match cache::Token::validate(cache, auth_header).await {
            Ok(()) => String::from(auth_header),
            Err(err) => {
                warn!("validate token failed: {}", err);
                String::new()
            }
        };

        let token = decode_token(&auth_header, signing_key).await;
        debug!("user requested with token {token:?}");

        let last_time = token.exp - Local::now().timestamp();

        let token_tracker = TokenTracker {
            token: Arc::new(Mutex::new(token.clone())),
            renew_requested: Arc::new(AtomicBool::new(last_time > 0 && last_time < buffer_time)),
            original: Some(auth_header.clone()),
        };

        req.extensions_mut().insert(token_tracker.clone());
        req.extensions_mut().insert(token.clone());

        let mut resp = next.run(req).await;

        if token_tracker
            .renew_requested
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            cache::Token::revoke(cache, &auth_header).await.ok();
            let old_token = token_tracker.token.lock().await.clone();
            let new_token = distribute_token(old_token, signing_key, expires_time).await;
            cache::Token::store(cache, token.id, &new_token)
                .await
                .map_err(|err| {
                    error!("failed to store new token: {:?}", err);
                })
                .ok();
            resp.headers_mut().insert(
                "Set-Token",
                new_token.parse().expect("failed to parse token"),
            );
        }

        Ok(resp)
    } else {
        req.extensions_mut().insert(Token::default());
        req.extensions_mut().insert(TokenTracker {
            token: Arc::new(Mutex::new(Token::default())),
            renew_requested: Arc::new(AtomicBool::new(false)),
            original: None,
        });
        Ok(next.run(req).await)
    }
}

pub async fn init_token_or_permission_required<B>(
    State(config): State<GlobalConfig>,
    Extension(token): Extension<Token>,
    platform_info: Option<Extension<ConfigModel>>,
    req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    if platform_info.is_some() {
        return Err((StatusCode::CONFLICT, "already configured"));
    }
    let init_token = req
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "))
        .map(|token| token.to_owned());
    debug!("user init token is: {:?}", init_token);
    match init_token {
        Some(token) => {
            debug!(
                "platform init token is: {}",
                config.server.init_token.trim()
            );
            if token.trim() == config.server.init_token.trim() {
                Ok(next.run(req).await)
            } else {
                Err((StatusCode::FORBIDDEN, "permission denied"))
            }
        }
        #[allow(clippy::redundant_closure_call)]
        None => permission_required!(Permission::Devops)(Extension(token), req, next).await,
    }
}

pub async fn challenge_privilege_required<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    Ok(next.run(req).await)
}

pub async fn game_privilege_required<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    Ok(next.run(req).await)
}

pub async fn game_challenges_privilege_required<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    Ok(next.run(req).await)
}

/// Construct a middleware closure that validate permissions from token.
///
/// Usage:
///
/// ```
/// Router::new()
///     .route(...)
///     .route_layer(axum::middleware::from_fn(permission_required!("basic", "verified", ...)))
/// ```
macro_rules! permission_required {
    ($($perm:expr),*) => {
        |
            axum::extract::Extension(token): axum::extract::Extension<crate::controller::layer::auth::Token>,
            req: axum::http::Request<_>,
            next: axum::middleware::Next<_>,
        | async move {
            let required_perms = [$($perm),*];
            tracing::debug!("user permissions: {:?}", token.permissions.0);
            tracing::debug!("required perms: {:?}", required_perms);
            match required_perms.iter().any(|perm| token.permissions.0.contains(perm)) {
                true => Ok(next.run(req).await),
                false => Err((axum::http::StatusCode::FORBIDDEN, "permission denied"))
            }
        }
    };
}

pub(crate) use permission_required;
