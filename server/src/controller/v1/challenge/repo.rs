use axum::{
    extract::{Multipart, Query, State},
    response::IntoResponse,
    routing::get,
    Extension, Json, Router,
};
use hyper::StatusCode;
use serde::Deserialize;
use tracing::error;

use crate::{bucket, config::GlobalConfig, controller::GlobalState, entity::challenge};

/*
 * Repo router
 *
 * user does not have any permissions, so this router if fully under admin
 * layer.
 */
pub fn router(_state: &GlobalState) -> Router<GlobalState> {
    Router::new().route(
        "/",
        get(get_file_list)
            .post(upload_attachment)
            .delete(delete_attachment),
    )
}

async fn get_file_list(
    State(config): State<GlobalConfig>, Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let files = bucket::get_static_attachment_list(&config, &challenge)
        .await
        .map_err(|err| {
            error!("failed to get static attachment list: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to get static attachment list",
            )
        })?;
    Ok(Json(files))
}

async fn upload_attachment(
    State(config): State<GlobalConfig>, Extension(challenge): Extension<challenge::Model>,
    multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    bucket::upload_static_attachment(&config, &challenge, multipart)
        .await
        .map_err(|err| {
            error!("failed to upload static attachment: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to upload static attachment",
            )
        })?;
    Ok(StatusCode::CREATED)
}

#[derive(Deserialize, Debug)]
struct DeleteAttachmentQuery {
    file: String,
}

async fn delete_attachment(
    State(config): State<GlobalConfig>, Extension(challenge): Extension<challenge::Model>,
    query: Query<DeleteAttachmentQuery>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    bucket::delete_file_by_hash(&config, &challenge, &query.file)
        .await
        .map_err(|err| {
            error!("failed to delete static attachment: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to delete static attachment",
            )
        })?;
    Ok(StatusCode::OK)
}
