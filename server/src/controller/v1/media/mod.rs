use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::{HeaderMap, StatusCode},
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use axum_extra::{headers::Host, TypedHeader};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tokio_util::io::ReaderStream;
use tracing::{error, warn};

use crate::{
    cache::manager::RedisPool,
    config::GlobalConfig,
    controller::{
        layer::auth::{permission_required_all, Token},
        GlobalState,
    },
    entity::{
        config::Model as ConfigModel,
        media::{create_media, Model as MediaModel},
        user::Permission,
    },
    media::{self, get_media},
};

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .route("/upload", post(upload_media))
        // media image size limited to 10MB
        .route_layer(DefaultBodyLimit::max(1024 * 1024 * 8))
        .route_layer(middleware::from_fn(permission_required_all!(
            Permission::Verified
        )))
        .route("/*path", get(download_media))
}

#[derive(Deserialize)]
struct UploadMediaQuery {
    require_thumbnail: Option<bool>,
}

#[derive(Serialize)]
struct UploadMediaResponse {
    model: MediaModel,
    remaining: i64,
}

async fn upload_media(
    State(config): State<GlobalConfig>, State(ref conn): State<DatabaseConnection>,
    State(ref _cache): State<RedisPool>, Extension(token): Extension<Token>,
    Extension(hot_config): Extension<ConfigModel>, Query(query): Query<UploadMediaQuery>,
    multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let require_thumbnail = query.require_thumbnail.unwrap_or(false);
    let upload_count = hot_config.media.limit + 1;
    // TODO: check and update user upload limit
    match media::save_media(
        &config.media.path,
        multipart,
        &format!("{}/temp", &config.media.path),
        require_thumbnail,
    )
    .await
    {
        Ok(mut model) => {
            model.uploader_id = token.id;
            match create_media(conn, model).await {
                Ok(model) => Ok(Json(UploadMediaResponse {
                    model,
                    remaining: hot_config.media.limit - upload_count,
                })),
                Err(err) => {
                    error!("Failed to save media: {}", err);
                    Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to save media"))
                }
            }
        }
        Err(err) => {
            error!("Failed to save media: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to save media"))
        }
    }
}

async fn download_media(
    State(config): State<GlobalConfig>, Extension(hot_config): Extension<ConfigModel>,
    Path(path): Path<String>, TypedHeader(host): TypedHeader<Host>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    // open localhost for local debugging
    if hot_config.media.anti_theft
        && host.hostname() != config.server.external_domain
        && host.hostname() != "localhost"
    {
        warn!(
            "Someone trying to get media source from host {}",
            host.hostname()
        );
        return Err((
            StatusCode::FORBIDDEN,
            "required content is not allowed to be accessed from this host",
        ));
    }
    let mime_type = mime_guess::from_path(&path).first_or_octet_stream();
    let file = match get_media(&config.media.path, &path).await {
        Ok(f) => f,
        Err(err) => {
            error!("Failed to get media: {}", err);
            return Err((StatusCode::NOT_FOUND, "resource not found"));
        }
    };

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        mime_type.to_string().parse().map_err(|err| {
            error!("Failed to parse mime type: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to parse mime type",
            )
        })?,
    );

    Ok((StatusCode::OK, headers, body))
}
