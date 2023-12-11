use axum::{
    extract::{Path, State},
    middleware,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use hyper::StatusCode;
use sea_orm::{DatabaseConnection, DbErr};
use tracing::{error, warn};

use crate::{
    controller::{layer::auth::permission_required_all, GlobalState},
    entity::{tag, user::Permission},
};

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .route("/", post(create_tag))
        .route("/:id", delete(delete_tag))
        .route_layer(middleware::from_fn(permission_required_all!(
            Permission::Organize
        )))
        .route("/", get(get_tag_list))
}

async fn get_tag_list(
    State(ref conn): State<DatabaseConnection>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match tag::get_tag_list(conn).await {
        Ok(tags) => Ok(Json(tags)),
        Err(err) => {
            error!("get_tag_list error: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to get tag list"))
        }
    }
}

async fn create_tag(
    State(ref conn): State<DatabaseConnection>, Json(tag): Json<tag::Model>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match tag::create_tag(conn, tag).await {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(err) => {
            error!("create_tag error: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to create tag"))
        }
    }
}

async fn delete_tag(
    State(ref conn): State<DatabaseConnection>, Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match tag::delete_tag(conn, id).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(DbErr::RecordNotFound(_)) => Err((StatusCode::NOT_FOUND, "tag not found")),
        Err(DbErr::Custom(_)) => {
            warn!("try to delete tag {} used by challenge", id);
            Err((StatusCode::BAD_REQUEST, "tag is used by challenge"))
        }
        Err(err) => {
            error!("delete_tag error: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to delete tag"))
        }
    }
}
