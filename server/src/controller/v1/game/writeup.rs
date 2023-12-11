use axum::{
    extract::{Query, State},
    middleware,
    response::IntoResponse,
    routing::{get, patch},
    Extension, Json, Router,
};
use hyper::StatusCode;
use sea_orm::{DatabaseConnection, DbErr};
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

use crate::{
    controller::{
        layer::{auth, info},
        GlobalState,
    },
    entity::{
        game,
        user::{self, Permission},
        write_up,
    },
};

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .route("/audit", patch(audit_writeup))
        .route_layer(middleware::from_fn(auth::permission_required_all!(
            Permission::Audit
        )))
        .route("/", get(get_writeup_list).post(submit_writeup))
        .route("/detail", get(get_writeup))
        .route(
            "/self",
            get(get_self_writeup)
                .patch(update_self_writeup)
                .delete(delete_self_writeup),
        )
        .route_layer(middleware::from_fn_with_state(
            _state.clone(),
            info::prepare_user_full_info,
        ))
        .route_layer(middleware::from_fn_with_state(
            _state.clone(),
            info::prepare_game_info_in_path,
        ))
        .route_layer(middleware::from_fn(auth::permission_required_all!(
            Permission::Verified
        )))
}

#[derive(Deserialize)]
struct WriteupIDQuery {
    pub writeup_id: i64,
}

#[derive(Deserialize)]
struct WriteupListQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Serialize)]
struct WriteupList {
    pub writeups: Vec<write_up::ModelOnlyTeamInfo>,
    pub total: u64,
}

async fn get_writeup_list(
    State(ref conn): State<DatabaseConnection>, Extension(game): Extension<game::Model>,
    Extension(user): Extension<user::Model>, Query(params): Query<WriteupListQuery>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(10);
    if page < 1 || per_page < 1 {
        error!("Invalid page={} or per_page={}", page, per_page);
        return Err((StatusCode::BAD_REQUEST, "invalid parameters"));
    }
    let is_admin = user.permissions.0.iter().any(|p| {
        matches!(
            p,
            &Permission::Organize | &Permission::Devops | &Permission::Audit
        )
    });
    if !is_admin && !game.end_and_archive() {
        return Err((StatusCode::FORBIDDEN, "cannot see writeup in this time"));
    }
    match write_up::get_writeup_page(conn, game.id, is_admin, page, per_page).await {
        Ok((writeups, total)) => Ok(Json(WriteupList { writeups, total })),
        Err(e) => {
            error!("Failed to get writeup list: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to get writeup list",
            ))
        }
    }
}

async fn submit_writeup(
    State(ref conn): State<DatabaseConnection>, Extension(user): Extension<user::Model>,
    Extension(game): Extension<game::Model>, Json(data): Json<write_up::Model>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    if !user.permissions.0.iter().any(|p| {
        matches!(
            p,
            &Permission::Organize | &Permission::Devops | &Permission::Audit
        )
    }) && !game.end_but_not_archive()
    {
        return Err((StatusCode::FORBIDDEN, "cannot submit writeup in this time"));
    }
    match write_up::create_writeup(conn, user.id, game.id, data).await {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(DbErr::RecordNotInserted) => {
            warn!(
                "user {} try to submit multiple writeup for same game",
                user.id
            );
            Err((
                StatusCode::BAD_REQUEST,
                "can only create one writeup for same game",
            ))
        }
        Err(e) => {
            error!("Failed to create writeup: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to submit writeup",
            ))
        }
    }
}

async fn get_writeup(
    State(ref conn): State<DatabaseConnection>, Extension(user): Extension<user::Model>,
    Extension(game): Extension<game::Model>, Query(query): Query<WriteupIDQuery>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let writeup_id = query.writeup_id;
    let is_admin = user.permissions.0.iter().any(|p| {
        matches!(
            p,
            &Permission::Organize | &Permission::Devops | &Permission::Audit
        )
    });
    if !is_admin && !game.end_and_archive() {
        return Err((StatusCode::FORBIDDEN, "cannot see writeup in this time"));
    }
    match write_up::get_writeup_detail(conn, writeup_id, !is_admin).await {
        Ok(Some(writeup)) => Ok(Json(writeup)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "writeup not found")),
        Err(DbErr::RecordNotFound(_)) => Err((StatusCode::NOT_FOUND, "writeup not found")),
        Err(e) => {
            error!("Failed to get writeup: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to get writeup"))
        }
    }
}

async fn get_self_writeup(
    State(ref conn): State<DatabaseConnection>, Extension(user): Extension<user::Model>,
    Extension(game): Extension<game::Model>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match write_up::get_writeup_by_game_and_user_id(conn, game.id, user.id).await {
        Ok(Some(writeup)) => Ok(Json(writeup)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "writeup not found")),
        Err(e) => {
            error!("Failed to get writeup: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to get writeup"))
        }
    }
}

pub async fn update_self_writeup(
    State(ref conn): State<DatabaseConnection>, Extension(user): Extension<user::Model>,
    Extension(game): Extension<game::Model>, Json(data): Json<write_up::Model>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    if game.end_and_archive() {
        return Err((StatusCode::FORBIDDEN, "cannot update writeup in this time"));
    }
    match write_up::update_writeup(conn, game.id, user.id, data).await {
        Ok(writeup) => Ok(Json(writeup)),
        Err(DbErr::RecordNotFound(_)) => Err((StatusCode::NOT_FOUND, "writeup not found")),
        Err(e) => {
            error!("Failed to update writeup: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update writeup",
            ))
        }
    }
}

pub async fn delete_self_writeup(
    State(ref conn): State<DatabaseConnection>, Extension(user): Extension<user::Model>,
    Extension(game): Extension<game::Model>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    if game.end_and_archive() {
        return Err((StatusCode::FORBIDDEN, "cannot delete writeup in this time"));
    }
    match write_up::delete_writeup_by_game_and_user_id(conn, game.id, user.id).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(DbErr::RecordNotFound(_)) => Err((StatusCode::NOT_FOUND, "writeup not found")),
        Err(e) => {
            error!("Failed to delete writeup: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete writeup",
            ))
        }
    }
}

#[derive(Deserialize)]
struct WriteupAduitQuery {
    pub writeup_id: Option<i64>,
    pub audit: Option<bool>,
}

async fn audit_writeup(
    State(ref conn): State<DatabaseConnection>, Query(params): Query<WriteupAduitQuery>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let writeup_id = match params.writeup_id {
        Some(id) => id,
        None => return Err((StatusCode::BAD_REQUEST, "writeup_id is required")),
    };
    let audit = match params.audit {
        Some(audit) => audit,
        None => return Err((StatusCode::BAD_REQUEST, "audit is required")),
    };
    match write_up::audit_writeup(conn, writeup_id, !audit).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(DbErr::RecordNotFound(_)) => Err((StatusCode::NOT_FOUND, "writeup not found")),
        Err(e) => {
            error!("Failed to update writeup: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update writeup",
            ))
        }
    }
}
