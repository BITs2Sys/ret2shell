use axum::{
    extract::{Path, Query, State},
    middleware,
    response::IntoResponse,
    routing::{get, patch, post},
    Extension, Json, Router,
};
use hyper::StatusCode;
use sea_orm::{DatabaseConnection, DbErr};
use serde::Deserialize;
use tracing::error;

use crate::{
    controller::{
        layer::auth::{self, Token},
        GlobalState,
    },
    entity::{
        user::Permission,
        wiki::{self, Model as WikiModel},
    },
};

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .route("/", post(create_wiki))
        .route("/:id", patch(update_wiki).delete(delete_wiki))
        .route_layer(middleware::from_fn(auth::permission_required_all!(
            Permission::Publish
        )))
        .route("/:id", get(get_wiki))
        .route("/", get(get_wiki_list))
}

#[derive(Deserialize)]
struct ListParams {
    parent_id: Option<i64>,
}

async fn get_wiki_list(
    State(ref conn): State<DatabaseConnection>, Query(params): Query<ListParams>,
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
    State(ref conn): State<DatabaseConnection>, Extension(token): Extension<Token>,
    Json(data): Json<WikiModel>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let data = WikiModel {
        author_id: Some(token.id),
        ..data
    };
    match wiki::create_wiki(conn, data).await {
        Ok(data) => Ok((StatusCode::CREATED, Json(data))),
        Err(err) => {
            error!("Failed to create wiki: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to create wiki"))
        }
    }
}

async fn update_wiki(
    State(ref conn): State<DatabaseConnection>, Path(id): Path<i64>, Json(data): Json<WikiModel>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match wiki::update_wiki(conn, id, data).await {
        Ok(data) => Ok((StatusCode::OK, Json(data))),
        Err(DbErr::RecordNotFound(_)) => Err((StatusCode::NOT_FOUND, "wiki not found")),
        Err(err) => {
            error!("Failed to update wiki: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to update wiki"))
        }
    }
}

async fn delete_wiki(
    State(ref conn): State<DatabaseConnection>, Path(id): Path<i64>,
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
    State(ref conn): State<DatabaseConnection>, Path(id): Path<i64>,
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
