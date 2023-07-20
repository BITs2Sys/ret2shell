//! The controller, which is used to handle the HTTP requests.
//!
//!

use axum::{
    body::Body,
    extract::FromRef,
    http::{HeaderValue, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use sea_orm::DatabaseConnection;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{info, Span};

use crate::{audit::Auditor, config::GlobalConfig};

mod account;
mod announcement;
mod certificate;
mod challenge;
mod game;
mod media;
mod middleware;
mod platform;
mod user;

use middleware::forwarded::get_client_ip;

#[derive(Clone, FromRef)]
pub struct GlobalState {
    pub db: DatabaseConnection,
    pub auditor: Auditor,
}

pub async fn initialize(config: &GlobalConfig, state: GlobalState) -> anyhow::Result<Router> {
    let api_base_path = &config.server.api_base_path;
    let cors_origins = &config.server.cors_origins;
    let api_router = construct_router();
    let router = Router::new()
        .nest(&api_base_path, api_router)
        .layer(
            CorsLayer::new()
                .allow_headers(Any)
                .allow_methods(Any)
                .allow_origin(cors_origins.parse::<HeaderValue>().unwrap()),
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
        .with_state(Arc::new(state));
    Ok(router)
}

fn construct_router() -> Router<Arc<GlobalState>> {
    Router::new()
        .nest("/account", account::router())
        .nest("/announcement", announcement::router())
        .nest("/certificate", certificate::router())
        .nest("/challenge", challenge::router())
        .nest("/game", game::router())
        .nest("/media", media::router())
        .nest("/platform", platform::router())
        .nest("/user", user::router())
        .route("/ping", get(ping))
}

async fn ping() -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    Ok("pong")
}
