use axum::{
  extract::{DefaultBodyLimit, Multipart, Path, State},
  middleware,
  response::IntoResponse,
  routing::get,
  Extension, Json, Router,
};
use futures::TryStreamExt;
use r2s_cache::Cache;
use r2s_cluster::Cluster;
use r2s_config::GlobalConfig;
use r2s_database::user::Permission;
use r2s_event::{
  events::{DevopsEvent, DevopsEventType, EventContainer},
  Event,
};
use r2s_queue::Queue;
use tokio_util::io::StreamReader;
use tracing::{debug, error, info, warn};

use crate::{
  middleware::auth::{self, Token},
  traits::{GlobalState, ResponseError},
};

pub fn router(state: &GlobalState) -> Router<GlobalState> {
  if state.config.cluster.as_ref().is_some_and(|c| c.enabled) {
    let cluster = state.cluster.clone();
    let queue = state.queue.clone();
    tokio::spawn(cluster_maintain_worker(cluster, queue));
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
    .route("/calmdown", get(get_calmdown_status))
    .route_layer(middleware::from_fn(auth::permission_required_all!(
      Permission::Basic,
      Permission::Verified
    )))
}

async fn cluster_maintain_worker(cluster: Cluster, queue: Queue) {
  let mut overloaded = false;
  loop {
    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    debug!("Checking outdated pods...");
    match cluster
      .at("ret2shell-challenge")
      .delete_outdated_pods()
      .await
    {
      Ok((o, running, pending)) => {
        if o {
          warn!(
            "Cluster is overloaded: running={}, pending={}",
            running, pending
          );
          let event = EventContainer {
            game_id: 0,
            event: Event::Devops(DevopsEvent {
              event_type: DevopsEventType::ClusterOverloaded,
              running: running as i64,
              pending: pending as i64,
            }),
          };
          queue.publish("event", event).await.ok();
        } else if !o && overloaded != o {
          info!("Cluster is recovered");
          let event = EventContainer {
            game_id: 0,
            event: Event::Devops(DevopsEvent {
              event_type: DevopsEventType::ClusterRecovered,
              running: running as i64,
              pending: pending as i64,
            }),
          };
          queue.publish("event", event).await.ok();
        }
        overloaded = o;
      }
      Err(err) => error!("Failed to delete outdated pods: {:?}", err),
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
  State(cluster): State<Cluster>, State(cache): State<Cache>,
) -> Result<impl IntoResponse, ResponseError> {
  let repos: Option<Vec<String>> = cache.at("registry").get("repos").await?;
  if let Some(repos) = repos {
    return Ok(Json(repos));
  }
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
  State(cluster): State<Cluster>, State(cache): State<Cache>, mut multipart: Multipart,
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
    cache.at("registry").del("repos").await?;
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

async fn get_calmdown_status(
  State(cache): State<Cache>, Extension(token): Extension<Token>,
) -> Result<impl IntoResponse, ResponseError> {
  let timestamp = cache.at("cluster").get::<i64>(token.id).await?;
  Ok(Json(timestamp))
}
