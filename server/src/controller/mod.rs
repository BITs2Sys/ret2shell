//! The controller, which is used to handle the HTTP requests.

use std::{net::IpAddr, time::Duration};

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
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{info, Span};

use crate::{audit::Auditor, cache::manager::RedisPool, config::GlobalConfig, license::License};

mod layer;
mod v1;

use layer::{
    auth::extract_user_info,
    forwarded::{get_client_ip, record_ip_address},
};

#[derive(Clone, FromRef)]
pub struct GlobalState {
    pub config: GlobalConfig,
    pub db: DatabaseConnection,
    pub cache: RedisPool,
    pub auditor: Auditor,
    pub queue: async_nats::jetstream::Context,
    pub cluster: kube::Client,
    pub license: License,
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
        .nest("/v1", v1::router(state))
        .route("/ping", get(ping))
        .route_layer(from_fn_with_state(state.clone(), record_ip_address))
        .route_layer(from_fn_with_state(state.clone(), extract_user_info))
}

async fn ping() -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    Ok("pong")
}
