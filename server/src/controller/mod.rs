//! The controller, which is used to handle the HTTP requests.
//!
//!

use axum::{
    body::Body,
    extract::FromRef,
    http::{HeaderValue, Request, StatusCode},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use sea_orm::DatabaseConnection;
use std::net::IpAddr;
use std::time::Duration;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{info, Span};

use crate::{audit::Auditor, cache::manager::RedisPool, config::GlobalConfig};

mod account;
mod announcement;
mod calendar;
mod certificate;
mod challenge;
mod game;
mod instance;
mod layer;
mod media;
mod platform;
mod user;
mod wiki;

use layer::forwarded::get_client_ip;

use self::layer::auth::extract_user_info;

#[derive(Clone, FromRef)]
pub struct GlobalState {
    pub config: GlobalConfig,
    pub db: DatabaseConnection,
    pub cache: RedisPool,
    pub auditor: Auditor,
    pub queue: async_nats::Client,
}

pub async fn initialize(config: &GlobalConfig, state: GlobalState) -> anyhow::Result<Router> {
    let api_base_path = &config.server.api_base_path;
    let cors_origins = &config.server.cors_origins;
    let api_router = construct_router(&state);
    let router = Router::new()
        .nest(api_base_path, api_router)
        .route_layer(from_fn_with_state(
            state.clone(),
            layer::info::prepare_platform_info,
        ))
        .layer(
            CorsLayer::new()
                .allow_headers(Any)
                .allow_methods(Any)
                .allow_origin(
                    cors_origins
                        .parse::<HeaderValue>()
                        .expect("invalid CORS origins"),
                ),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    let ip = get_client_ip(request)
                        .unwrap_or(IpAddr::V4("0.0.0.0".parse().expect("Impossible!!!")));
                    tracing::info_span!("http",
                        from = %ip.to_string(),
                        method = %request.method(),
                        uri = %request.uri().path(),
                    )
                })
                .on_request(())
                .on_response(|response: &Response, latency: Duration, _span: &Span| {
                    info!("[{}] in {}ms", response.status(), latency.as_millis());
                }),
        )
        .with_state::<()>(state);
    Ok(router)
}

fn construct_router(state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .nest("/account", account::router(state))
        .nest("/announcement", announcement::router(state))
        .nest("/certificate", certificate::router(state))
        .nest("/game", game::router(state))
        .nest("/challenge", challenge::router(state))
        .nest("/media", media::router(state))
        .nest("/platform", platform::router(state))
        .nest("/user", user::router(state))
        .nest("/calendar", calendar::router(state))
        .nest("/wiki", wiki::router(state))
        .nest("/instance", instance::router(state))
        .route("/ping", get(ping))
        .route_layer(from_fn_with_state(state.clone(), extract_user_info))
}

async fn ping() -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    Ok("pong")
}
