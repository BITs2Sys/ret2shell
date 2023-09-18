use super::layer::{auth, info};
use crate::{
    controller::GlobalState,
    entity::user::{self, Permission},
};
use axum::{
    extract::{Path, Query, State},
    middleware,
    response::IntoResponse,
    routing::{get, patch},
    Extension, Json, Router,
};
use hyper::StatusCode;
use sea_orm::{DatabaseConnection, DbErr};
use serde::{Deserialize, Serialize};
use tracing::error;

mod institute;

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .route("/:id", patch(update_user).delete(delete_user))
        .route_layer(middleware::from_fn(auth::permission_required_all!(
            Permission::Devops
        )))
        .route("/", get(get_user_list))
        .route_layer(middleware::from_fn(auth::permission_required_all!(
            Permission::Verified
        )))
        .route("/:id", get(get_user_info))
        .route_layer(middleware::from_fn_with_state(
            _state.clone(),
            info::prepare_user_full_info,
        ))
}

#[derive(Deserialize)]
struct ListParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub order: Option<String>,
    pub filter: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct UserList {
    pub users: Vec<user::Model>,
    pub total: u64,
}

async fn get_user_list(
    State(ref conn): State<DatabaseConnection>,
    Extension(op_user): Extension<user::Model>,
    Query(params): Query<ListParams>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(10);
    let filter = params.filter;
    let order = params.order;
    if page < 1 || per_page < 1 {
        error!("Invalid page={} or per_page={}", page, per_page);
        return Err((StatusCode::BAD_REQUEST, "invalid parameters"));
    }
    match user::get_user_page(conn, page, per_page, order, filter, &op_user).await {
        Ok((users, total)) => Ok(Json(UserList { users, total })),
        Err(err) => {
            error!("Failed to get user list: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to get user list"))
        }
    }
}

async fn get_user_info(
    State(ref conn): State<DatabaseConnection>,
    op_user: Option<Extension<user::Model>>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let user = match user::get_user(conn, id).await {
        Ok(user) => user,
        Err(DbErr::RecordNotFound(_)) => return Err((StatusCode::NOT_FOUND, "user not found")),
        Err(err) => {
            error!("Failed to get user info: {}", err);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to get user info"));
        }
    };
    if op_user.is_some_and(|Extension(op_user)| op_user.permissions.0.contains(&Permission::Devops) || op_user.id == user.id)
    {
        return Ok(Json(user));
    }
    if user.hidden {
        Err((StatusCode::NOT_FOUND, "user not found"))
    } else {
        Ok(Json(user.desensitize()))
    }
}

async fn update_user(
    State(ref conn): State<DatabaseConnection>,
    Path(id): Path<i64>,
    Json(user): Json<user::Model>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match user::update_user(conn, id, user).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(DbErr::RecordNotFound(_)) => Err((StatusCode::NOT_FOUND, "user not found")),
        Err(err) => {
            error!("Failed to update user: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to update user"))
        }
    }
}

async fn delete_user(
    State(ref conn): State<DatabaseConnection>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match user::delete_user(conn, id).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(DbErr::RecordNotFound(_)) => Err((StatusCode::NOT_FOUND, "user not found")),
        Err(err) => {
            error!("Failed to delete user: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to delete user"))
        }
    }
}
