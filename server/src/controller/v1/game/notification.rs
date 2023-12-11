use axum::{
    extract::{Path, Query, State},
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use hyper::StatusCode;
use sea_orm::{DatabaseConnection, DbErr};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::{
    controller::{layer::auth, GlobalState},
    entity::{notification, user::Permission},
};

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .route("/", post(create_notification).delete(delete_notification))
        .route_layer(middleware::from_fn(auth::permission_required_all!(
            Permission::Organize
        )))
        .route("/", get(get_notification_list))
}

#[derive(Deserialize)]
struct NotificationIDQuery {
    pub notification_id: i64,
}

async fn create_notification(
    State(ref conn): State<DatabaseConnection>, Path(game_id): Path<i64>,
    Json(notification): Json<notification::Model>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    debug!("game_id={}", game_id);
    match notification::create_notification(conn, game_id, notification).await {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(DbErr::RecordNotFound(_)) => Err((StatusCode::NOT_FOUND, "game not found")),
        Err(e) => {
            error!("Failed to create notification: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create notification",
            ))
        }
    }
}

#[derive(Deserialize)]
struct NotificationListQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Serialize)]
struct NotificationList {
    pub notifications: Vec<notification::Model>,
    pub total: u64,
}

async fn get_notification_list(
    State(ref conn): State<DatabaseConnection>, Query(params): Query<NotificationListQuery>,
    Path(game_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(10);
    if page < 1 || per_page < 1 {
        error!("Invalid page={} or per_page={}", page, per_page);
        return Err((StatusCode::BAD_REQUEST, "invalid parameters"));
    }
    match notification::get_notification_page(conn, page, per_page, game_id).await {
        Ok((notifications, total)) => Ok(Json(NotificationList {
            notifications,
            total,
        })),
        Err(e) => {
            error!("Failed to get notification list: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to get notification list",
            ))
        }
    }
}

async fn delete_notification(
    State(ref conn): State<DatabaseConnection>, Query(query): Query<NotificationIDQuery>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let notification_id = query.notification_id;
    match notification::delete_notification(conn, notification_id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(DbErr::RecordNotFound(_)) => Err((StatusCode::NOT_FOUND, "notification not found")),
        Err(e) => {
            error!("Failed to delete notification: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete notification",
            ))
        }
    }
}
