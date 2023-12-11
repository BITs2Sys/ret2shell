use axum::{
    extract::{Path, State, WebSocketUpgrade},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::get,
    Extension, Json, Router,
};
use sea_orm::DatabaseConnection;
use tracing::error;

use crate::{
    controller::{
        layer::auth::{permission_required_all, Token},
        GlobalState,
    },
    entity::{
        instance::{get_instance_by_user_id, get_instance_by_wsrx},
        user::Permission,
    },
    traffic,
};

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .route("/self", get(get_self_running_instance))
        .route_layer(middleware::from_fn(permission_required_all!(
            Permission::Basic,
            Permission::Verified
        )))
        .route("/:instance", get(start_proxy).options(ping))
}

async fn get_self_running_instance(
    State(ref db): State<DatabaseConnection>, Extension(token): Extension<Token>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let instance = get_instance_by_user_id(db, token.id)
        .await
        .map_err(|err| {
            error!("get user current instance failed: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "encountered database error",
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "instance not found"))?;
    Ok(Json(instance.desentialize()))
}

async fn start_proxy(
    State(ref conn): State<DatabaseConnection>, Path(instance): Path<String>,
    ws: Option<WebSocketUpgrade>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let inst = get_instance_by_wsrx(conn, &instance)
        .await
        .map_err(|e| {
            error!("get instance by wsrx error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "get instance by wsrx error",
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "instance not found"))?;

    if !inst.running || inst.should_stop() {
        return Err((StatusCode::FORBIDDEN, "instance is down"));
    }

    if ws.is_none() {
        return Err((StatusCode::NO_CONTENT, ""));
    }

    let ws = ws.unwrap();
    Ok(ws.on_upgrade(|socket| async move {
        match traffic::connect(inst.addr, socket).await {
            Ok(_) => {}
            Err(e) => {
                error!("proxying websocket error: {}", e);
            }
        };
    }))
}

async fn ping() -> &'static str {
    "pong"
}
