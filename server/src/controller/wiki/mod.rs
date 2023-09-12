use super::{layer::auth, GlobalState};
use crate::entity::user::Permission;
use crate::entity::wiki::{self, Model as WikiModel};
use crate::entity::wiki_related::{self, Model as RelatedWikiModel};
use axum::{
    extract::{Path, Query, State},
    middleware,
    response::IntoResponse,
    routing::{get, patch, post},
    Json, Router,
};
use hyper::StatusCode;
use sea_orm::{DatabaseConnection, DbErr};
use serde::{Deserialize, Serialize};
use tracing::error;

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .route("/", post(create_wiki))
        .route(
            "/related",
            post(add_related_record).delete(delete_related_record),
        )
        .route("/:id", patch(update_wiki).delete(delete_wiki))
        .route_layer(middleware::from_fn(auth::permission_required!(
            Permission::Publish
        )))
        .route("/:id/related", get(get_related_wiki_list))
        .route("/:id", get(get_wiki))
        .route("/", get(get_wiki_list))
}

#[derive(Deserialize)]
struct ListParams {
    parent_id: Option<i64>
}

async fn get_wiki_list(
    State(ref conn): State<DatabaseConnection>,
    Query(params): Query<ListParams>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match wiki::get_wiki_page(conn, params.parent_id).await {
        Ok(wikis) => Ok(Json(wikis)),
        Err(err) => {
            error!("Failed to get wiki list: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to get wiki list"))
        }
    }
}

async fn create_wiki(
    State(ref conn): State<DatabaseConnection>,
    Json(data): Json<WikiModel>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match wiki::create_wiki(conn, data).await {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(err) => {
            error!("Failed to create wiki: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to create wiki"))
        }
    }
}

async fn update_wiki(
    State(ref conn): State<DatabaseConnection>,
    Path(id): Path<i64>,
    Json(data): Json<WikiModel>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match wiki::update_wiki(conn, id, data).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(DbErr::RecordNotFound(_)) => Err((StatusCode::NOT_FOUND, "wiki not found")),
        Err(err) => {
            error!("Failed to update wiki: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to update wiki"))
        }
    }
}

async fn delete_wiki(
    State(ref conn): State<DatabaseConnection>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match wiki::delete_wiki(conn, id).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(DbErr::RecordNotFound(_)) => Err((StatusCode::NOT_FOUND, "wiki not found")),
        Err(err) => {
            error!("Failed to delete wiki: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to delete wiki"))
        }
    }
}

async fn get_wiki(
    State(ref conn): State<DatabaseConnection>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match wiki::get_wiki_by_id(conn, id).await {
        Ok(Some(wiki)) => Ok(Json(wiki)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "wiki not found")),
        Err(err) => {
            error!("Failed to get wiki: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to get wiki"))
        }
    }
}

#[derive(Serialize, Deserialize)]
struct RelatedWikiList {
    wikis: Vec<RelatedWikiModel>,
    total: u64,
}

async fn get_related_wiki_list(
    State(ref conn): State<DatabaseConnection>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match wiki_related::get_related_wiki_by_wiki_id(conn, id).await {
        Ok((wikis, total)) => Ok(Json(RelatedWikiList { wikis, total })),
        Err(DbErr::RecordNotFound(_)) => Err((StatusCode::NOT_FOUND, "wiki not found")),
        Err(err) => {
            error!("Failed to get related wiki list: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to get related wiki list",
            ))
        }
    }
}

async fn add_related_record(
    State(ref conn): State<DatabaseConnection>,
    Json(data): Json<RelatedWikiModel>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match wiki_related::add_related_record(conn, data).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(DbErr::RecordNotUpdated) => {
            Err((StatusCode::BAD_REQUEST, "related record already exists"))
        }
        Err(err) => {
            error!("Failed to add related record: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to add related record",
            ))
        }
    }
}

async fn delete_related_record(
    State(ref conn): State<DatabaseConnection>,
    Json(data): Json<RelatedWikiModel>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match wiki_related::delete_related_record(conn, data).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(DbErr::RecordNotFound(_)) => Err((StatusCode::NOT_FOUND, "wiki not found")),
        Err(err) => {
            error!("Failed to delete related record: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to delete related record",
            ))
        }
    }
}
