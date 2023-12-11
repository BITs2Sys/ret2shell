use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, patch, post},
    Json, Router,
};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    controller::{layer::auth, GlobalState},
    entity::{
        announcement::{self, Model as AnnouncementModel},
        user::Permission,
    },
};

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .route("/", post(create_announcement))
        .route(
            "/:id",
            patch(update_announcement).delete(delete_announcement),
        )
        .route_layer(middleware::from_fn(auth::permission_required_all!(
            Permission::Publish
        )))
        .route("/:id", get(get_announcement))
        .route("/", get(get_announcement_list))
}

#[derive(Deserialize)]
struct ListParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct AnnouncementList {
    pub announcements: Vec<AnnouncementModel>,
    pub total: u64,
}

async fn get_announcement_list(
    State(ref conn): State<DatabaseConnection>, Query(params): Query<ListParams>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(10);
    if page < 1 || per_page < 1 {
        error!("Invalid page={} or per_page={}", page, per_page);
        return Err((StatusCode::BAD_REQUEST, "invalid parameters"));
    }
    match announcement::get_announcement_page(conn, page, per_page).await {
        Ok((announcements, total)) => Ok(Json(AnnouncementList {
            announcements,
            total,
        })),
        Err(err) => {
            error!("Failed to get announcement list: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to get announcement list",
            ))
        }
    }
}

async fn get_announcement(
    State(ref conn): State<DatabaseConnection>, Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match announcement::get_announcement_by_id(conn, id).await {
        Ok(Some(announcement)) => Ok(Json(announcement)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "announcement not found")),
        Err(err) => {
            error!("Failed to get announcement: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to get announcement",
            ))
        }
    }
}

async fn create_announcement(
    State(ref conn): State<DatabaseConnection>, Json(data): Json<AnnouncementModel>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match announcement::create_announcement(conn, data.clone()).await {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(err) => {
            error!("Failed to create announcement: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to create announcement",
            ))
        }
    }
    // TODO: push announcement released event
    // let data = AnnouncementReleasedEventData {
    //     is_created: true,
    //     title: data.title,
    // };
    // let _ = push_announcement_released_event(PusherEventLevel::Public, data,
    // cache.pusher).await;
}

async fn update_announcement(
    State(ref conn): State<DatabaseConnection>, Path(id): Path<i64>,
    Json(data): Json<AnnouncementModel>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match announcement::update_announcement(conn, id, data.clone()).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(err) => {
            error!("Failed to update announcement: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to update announcement",
            ))
        }
    }
    // TODO: push announcement released event
    // let data = AnnouncementReleasedEventData {
    //     is_created: false,
    //     title: data.title,
    // };
    // let _ = push_announcement_released_event(PusherEventLevel::Public, data,
    // cache.pusher).await;
}

async fn delete_announcement(
    State(ref conn): State<DatabaseConnection>, Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match announcement::delete_announcement(conn, id).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(err) => {
            error!("Failed to delete announcement: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to delete announcement",
            ))
        }
    }
}
