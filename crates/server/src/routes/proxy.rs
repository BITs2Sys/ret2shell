use axum::{
  Router,
  body::Body,
  extract::{Request, State},
  http::Uri,
  response::IntoResponse,
  routing::{any, get},
};
use hyper_util::client::legacy::connect::HttpConnector;
use r2s_config::server::FrontendServeType;
use tower_http::services::{ServeDir, ServeFile};
use tracing::debug;

use crate::traits::{GlobalState, ResponseError};

type HTTPClient = hyper_util::client::legacy::Client<HttpConnector, Body>;
pub fn router(state: &GlobalState) -> Router<GlobalState> {
  let frontend_config = state.config.server.clone().unwrap_or_default().frontend;
  if let Some(frontend_config) = frontend_config {
    match frontend_config.serve_type {
      FrontendServeType::Static => Router::new().fallback_service(
        ServeDir::new(&frontend_config.path)
          .precompressed_gzip()
          .not_found_service(ServeFile::new(format!(
            "{}/index.html",
            frontend_config.path
          ))),
      ),
      FrontendServeType::Proxy => Router::new()
        .route("/", any(frontend_proxy_handler))
        .route("/{*path}", any(frontend_proxy_handler)),
    }
  } else {
    Router::new()
      .route("/", get(no_frontend_proxy_handler))
      .route("/{*path}", get(no_frontend_proxy_handler))
  }
}

async fn frontend_proxy_handler(
  State(state): State<GlobalState>, State(client): State<HTTPClient>, mut req: Request,
) -> Result<impl IntoResponse, ResponseError> {
  // debug!("Proxying frontend request: {:?}", req);
  let frontend_config = state.config.server.clone().unwrap_or_default().frontend;
  let frontend_path = match frontend_config {
    Some(frontend_config) => frontend_config.path,
    None => {
      return Err(ResponseError::PreconditionFailed(String::from(
        "Frontend proxy not set for ret2shell, please contact the website devops",
      )));
    }
  };
  let path = req.uri().path();
  let path_query = req
    .uri()
    .path_and_query()
    .map(|pq| pq.as_str())
    .unwrap_or(path);
  let uri = format!("{}/{}", frontend_path, path_query);
  *req.uri_mut() = Uri::try_from(uri).unwrap();
  req.headers_mut().remove("host");

  let resp = client
    .request(req)
    .await
    .map_err(|err| ResponseError::BadRequest(format!("frontend proxy failed: {}", err)))?
    .into_response();
  debug!("Proxying frontend request: {:?}", resp);
  Ok(resp)
}

async fn no_frontend_proxy_handler() -> Result<(), ResponseError> {
  Err(ResponseError::PreconditionFailed(String::from(
    "Frontend proxy not set for ret2shell, please contact the website devops",
  )))
}
