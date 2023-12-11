use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, patch, post},
    Json, Router,
};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use tracing::error;

use crate::{
    controller::{layer::auth, GlobalState},
    entity::{calendar, calendar::Model as CalendarModel, user::Permission},
};

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .route("/", post(create_calendar))
        .route("/:id", patch(update_calendar).delete(delete_calendar))
        .route_layer(middleware::from_fn(auth::permission_required_all!(
            Permission::Calendar
        )))
        .route("/:id", get(get_calendar))
        .route("/", get(get_calendar_list))
}

#[derive(Deserialize)]
struct ListParams {
    start_time: i64,
    end_time: i64,
}

async fn get_calendar_list(
    State(ref conn): State<DatabaseConnection>, Query(params): Query<ListParams>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match calendar::get_calendar_list(conn, params.start_time, params.end_time).await {
        Ok(calendars) => Ok(Json(calendars)),
        Err(err) => {
            error!("Failed to get calendar list: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to get calendar list",
            ))
        }
    }
}

async fn get_calendar(
    State(ref conn): State<DatabaseConnection>, Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match calendar::get_calendar_by_id(conn, id).await {
        Ok(calendar) => Ok(Json(calendar)),
        Err(err) => {
            error!("Failed to get calendar: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to get calendar"))
        }
    }
}

async fn create_calendar(
    State(ref conn): State<DatabaseConnection>, Json(data): Json<CalendarModel>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match calendar::create_calendar(conn, data).await {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(err) => {
            error!("Failed to create calendar: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to create calendar",
            ))
        }
    }
}

async fn update_calendar(
    State(ref conn): State<DatabaseConnection>, Path(id): Path<i64>,
    Json(data): Json<CalendarModel>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match calendar::update_calendar(conn, id, data).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(err) => {
            error!("Failed to update calendar: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to update calendar",
            ))
        }
    }
}

async fn delete_calendar(
    State(ref conn): State<DatabaseConnection>, Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match calendar::delete_calendar(conn, id).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(err) => {
            error!("Failed to delete calendar: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to delete calendar",
            ))
        }
    }
}
