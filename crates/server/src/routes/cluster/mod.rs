use axum::{
  extract::{DefaultBodyLimit, Multipart, Path, State},
  middleware,
  response::IntoResponse,
  routing::get,
  Json, Router,
};
use futures::TryStreamExt;
use r2s_cluster::Cluster;
use r2s_config::GlobalConfig;
use r2s_database::user::Permission;
use tokio_util::io::StreamReader;
use tracing::{debug, error};

use crate::{
  middleware::auth,
  traits::{GlobalState, ResponseError},
};

pub fn router(state: &GlobalState) -> Router<GlobalState> {
  if state.config.cluster.as_ref().is_some_and(|c| c.enabled) {
    let cluster = state.cluster.clone();
    tokio::spawn(cluster_maintain_worker(cluster));
  }
  Router::new()
    .nest(
      "/",
      Router::new()
        .route("/config", get(get_cluster_config))
        .route("/node", get(get_cluster_nodes))
        .route_layer(middleware::from_fn(auth::permission_required_all!(
          Permission::DevOps
        ))),
    )
    .nest(
      "/repo",
      Router::new()
        .route("/config", get(get_cluster_registry_config))
        .route("/", get(get_cluster_registry_repo).post(upload_image))
        .route_layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
        .route("/:image", get(get_cluster_registry_image))
        .route_layer(middleware::from_fn(auth::permission_required_all!(
          Permission::Game
        ))),
    )
    .route_layer(middleware::from_fn(auth::permission_required_all!(
      Permission::Basic,
      Permission::Verified
    )))
}

async fn cluster_maintain_worker(cluster: Cluster) {
  loop {
    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    debug!("Checking outdated pods...");
    if let Err(e) = cluster.delete_outdated_pods().await {
      error!("Failed to delete outdated pods: {}", e);
    }
  }
}

async fn get_cluster_config(
  State(ref cluster): State<Cluster>,
) -> Result<impl IntoResponse, ResponseError> {
  let configs = cluster.configs().await?;
  Ok(Json(configs))
}

async fn get_cluster_nodes(
  State(ref cluster): State<Cluster>,
) -> Result<impl IntoResponse, ResponseError> {
  let nodes = cluster.nodes().await?;
  Ok(Json(nodes))
}

async fn get_cluster_registry_repo(
  State(cluster): State<Cluster>,
) -> Result<impl IntoResponse, ResponseError> {
  let registry = if let Some(registry) = cluster.registry {
    registry
  } else {
    return Err(ResponseError::NotFound("registry".to_string()));
  };

  let repos = registry.repositories().await?;
  Ok(Json(repos))
}

async fn get_cluster_registry_image(
  State(cluster): State<Cluster>, Path(repo): Path<String>,
) -> Result<impl IntoResponse, ResponseError> {
  let registry = if let Some(registry) = cluster.registry {
    registry
  } else {
    return Err(ResponseError::NotFound("registry".to_string()));
  };
  let tags = registry.images(&repo).await?;
  Ok(Json(tags))
}

async fn upload_image(
  State(cluster): State<Cluster>, mut multipart: Multipart,
) -> Result<impl IntoResponse, ResponseError> {
  let registry = if let Some(registry) = cluster.registry {
    registry
  } else {
    return Err(ResponseError::NotFound("registry".to_string()));
  };
  if let Some(field) = multipart
    .next_field()
    .await
    .map_err(|err| ResponseError::BadRequest(err.to_string()))?
  {
    let file_name = field
      .file_name()
      .ok_or(ResponseError::BadRequest(
        "file name is required".to_owned(),
      ))?
      .to_owned();
    let reader =
      StreamReader::new(field.map_err(|multipart_error| {
        std::io::Error::new(std::io::ErrorKind::Other, multipart_error)
      }));
    registry.upload_image(&file_name, reader).await?;
    Ok(())
  } else {
    Err(ResponseError::BadRequest("no file".to_string()))
  }
}

async fn get_cluster_registry_config(
  State(config): State<GlobalConfig>,
) -> Result<impl IntoResponse, ResponseError> {
  if let Some(cluster) = config.cluster {
    Ok(Json(cluster.registry))
  } else {
    Ok(Json(None))
  }
}
