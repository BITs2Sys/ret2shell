use axum::{
  Extension, Router,
  extract::{Request, State},
  http::{HeaderMap, HeaderValue, Uri},
  middleware,
  response::IntoResponse,
  routing::any,
};
use r2s_config::GlobalConfig;
use r2s_database::{game, user::Permission};
use r2s_migrator::Database;
use tracing::{debug, info, warn};

use crate::{
  middleware::auth::{self, Token},
  traits::{GlobalState, HTTPClient, ResponseError},
};

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
  Router::new()
    .route("/", any(proxy_to_registry))
    .route("/{*path}", any(proxy_to_registry))
    .route_layer(middleware::from_fn(auth::permission_required_all!(
      Permission::Game
    )))
}

fn infer_origin(headers: &HeaderMap) -> Result<String, anyhow::Error> {
  let mut scheme = "http".to_string();
  let mut host = "".to_string();

  // directly return if x-forwarded-origin is present
  if let Some(origin) = headers
    .get("x-forwarded-origin")
    .and_then(|v| v.to_str().ok())
  {
    return Ok(origin.to_string());
  }

  // infer scheme and host

  if let Some(s) = headers
    .get("x-forwarded-proto")
    .and_then(|v| v.to_str().ok())
  {
    scheme = s.to_string();
  }

  if let Some(h) = headers.get("host").and_then(|v| v.to_str().ok()) {
    host = h.to_string();
  }

  if let Some(h) = headers
    .get("x-forwarded-host")
    .and_then(|v| v.to_str().ok())
  {
    host = h.to_string();
  }

  if let Some(uri_str) = headers.get("x-forwarded-uri").and_then(|v| v.to_str().ok())
    && let Ok(u) = Uri::try_from(uri_str)
  {
    if let Some(s) = u.scheme_str() {
      scheme = s.to_string();
    }
    if let Some(h) = u.host() {
      host = h.to_string();
    }
  }

  if !host.is_empty() {
    return Ok(format!("{}://{}", scheme, host));
  }

  Err(anyhow::anyhow!("Cannot extract host from request headers"))
}

async fn proxy_to_registry(
  State(config): State<GlobalConfig>, State(ref db): State<Database>,
  State(client): State<HTTPClient>, Extension(token): Extension<Token>, mut req: Request,
) -> Result<impl IntoResponse, ResponseError> {
  debug!(?req, "proxying frontend request");
  let registry_config = config.cluster.clone().and_then(|v| v.registry);
  let (protocol, registry_url) = match registry_config {
    Some(c) => {
      if c.insecure {
        ("http", c.server)
      } else {
        ("https", c.server)
      }
    }
    None => {
      return Err(ResponseError::PreconditionFailed(String::from(
        "internal registry is not enabled, please contact the website devops",
      )));
    }
  };
  let path = req.uri().path();
  let path_query = req
    .uri()
    .path_and_query()
    .map(|pq| pq.as_str())
    .unwrap_or(path);

  let path = path.trim_start_matches("/");
  let path_query = path_query.trim_start_matches("/");

  debug!("Proxying frontend path with query: /{path_query}");

  // if path is not starts with v2, not implemented yet
  if !path.starts_with("v2") {
    return Err(ResponseError::BadRequest(format!(
      "invalid registry path: {path}"
    )));
  }

  let is_auth_path = path.trim_matches('/') == "v2";
  if is_auth_path {
    let resp = axum::response::Response::builder()
      .status(200)
      .body("OK".to_string())
      .unwrap();
    return Ok(resp.into_response());
  }

  if !token.permissions.0.contains(&Permission::DevOps) {
    let repo = match path.strip_prefix("/v2/").unwrap().split('/').next() {
      Some(repo) => repo,
      None => {
        return Err(ResponseError::BadRequest(
          "invalid registry path".to_string(),
        ));
      }
    };

    let game = match game::get_by_bucket(&db.conn, repo).await? {
      Some(game) => game,
      None => {
        return Err(ResponseError::NotFound(format!(
          "game scope {repo} not found"
        )));
      }
    };

    if !game.admins.0.contains(&token.id) {
      warn!(?repo, "user is not allowed to access this game scope");
      return Err(ResponseError::Forbidden("access denied".to_string()));
    }
    info!(
      %repo,
      image=%path_query.strip_prefix(repo).unwrap_or(path_query),
      "game admin pushed to game scope",
    );
  } else {
    info!(
      image=%path_query,
      "devops operate registry",
    );
  }

  let uri = format!(
    "{}://{}/{}",
    protocol,
    registry_url.trim_matches('/'),
    path_query
  );
  debug!(?uri, "proxying to registry url");
  *req.uri_mut() = Uri::try_from(uri)
    .map_err(|err| ResponseError::BadRequest(format!("invalid registry uri: {err}")))?;
  //req.headers_mut().remove("host");

  let mut resp = client
    .request(req)
    .await
    .map_err(|err| ResponseError::BadRequest(format!("registry proxy failed: {err}")))?;

  // modify response headers to set correct origin, if has location header
  if let Some(location) = resp.headers().get("location").and_then(|v| v.to_str().ok()) {
    let origin = infer_origin(resp.headers()).map_err(|err| {
      ResponseError::BadRequest(format!("failed to infer origin for location header: {err}"))
    })?;
    let new_location = match Uri::try_from(location) {
      Ok(u) => {
        let path_and_query = u.path_and_query().map_or("", |pq| pq.as_str());
        format!("{}{}", origin, path_and_query)
      }
      Err(_) => {
        // If location is a relative path, just prepend the origin
        let p = if location.starts_with('/') {
          location.to_string()
        } else {
          format!("/{}", location)
        };
        format!("{}{}", origin, p)
      }
    };
    if let Ok(new_value) = HeaderValue::from_str(&new_location) {
      resp.headers_mut().insert("location", new_value);
    }
  }

  debug!(?resp, "proxying registry response");
  Ok(resp.into_response())
}
