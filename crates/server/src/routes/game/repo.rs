use axum::{
  Extension, Json,
  body::{Body, Bytes},
  extract::{Query, State},
  http::{
    HeaderMap, StatusCode,
    header::{CACHE_CONTROL, CONTENT_TYPE},
  },
  response::IntoResponse,
};
use futures::TryStreamExt;
use r2s_bucket::{Bucket, git::to_pkt_line};
use r2s_database::game;
use regex::Regex;
use serde::Deserialize;
use tokio_stream::StreamExt;
use tokio_util::io::{ReaderStream, StreamReader};
use tracing::error;

use crate::traits::ResponseError;

#[derive(Deserialize)]
pub(super) struct GameRepoGitQuery {
  pub path: Option<String>,
}

pub(super) async fn get_game_repo_git(
  State(ref bucket): State<Bucket>, Extension(game): Extension<game::Model>,
  Query(query): Query<GameRepoGitQuery>,
) -> Result<impl IntoResponse, ResponseError> {
  let game_bucket = bucket
    .at(
      game
        .bucket
        .clone()
        .ok_or(ResponseError::PreconditionFailed(
          "game bucket not found".to_owned(),
        ))?,
    )
    .await?;
  let path = match query.path {
    Some(path) => path,
    None => ".".to_owned(),
  };

  Ok(Json(game_bucket.git.list_objects(&path).await?))
}

#[derive(Clone, Deserialize)]
pub(super) struct InfoRefsQuery {
  pub service: String,
}

impl InfoRefsQuery {
  pub fn service_trimmed(&self) -> String {
    self.service.trim_start_matches("git-").to_owned()
  }
}

fn check_git_protocol_safe(protocol: impl AsRef<str>) -> bool {
  let re = Regex::new(r"^[0-9a-zA-Z]+=[0-9a-zA-Z]+(:[0-9a-zA-Z]+=[0-9a-zA-Z]+)*$").unwrap();
  re.is_match(protocol.as_ref())
}

fn get_protocol(headers: &HeaderMap) -> Result<String, ResponseError> {
  let protocol = headers.get("Git-Protocol");
  if let Some(protocol) = protocol {
    let protocol = protocol.to_str().map_err(|err| {
      error!("Invalid git protocol: {}", err);
      ResponseError::BadRequest("invalid git protocol".to_owned())
    })?;
    if check_git_protocol_safe(protocol) {
      Ok(protocol.to_owned())
    } else {
      Err(ResponseError::BadRequest("invalid git protocol".to_owned()))
    }
  } else {
    Ok("".to_owned())
  }
}

pub(super) async fn game_repo_info_refs(
  State(ref bucket): State<Bucket>, Extension(game): Extension<game::Model>,
  Query(query): Query<InfoRefsQuery>, headers: HeaderMap, body: Body,
) -> Result<impl IntoResponse, ResponseError> {
  let service = query.service_trimmed();
  let protocol = get_protocol(&headers)?;
  let mut headers = HeaderMap::new();

  headers.insert(
    CONTENT_TYPE,
    format!("application/x-git-{service}-advertisement")
      .parse()
      .unwrap(),
  );
  headers.insert(CACHE_CONTROL, "no-cache".parse().unwrap());

  let game_bucket = bucket
    .at(
      game
        .bucket
        .clone()
        .ok_or(ResponseError::PreconditionFailed(
          "game bucket not found".to_owned(),
        ))?,
    )
    .await?;

  let stream_reader = StreamReader::new(body.into_data_stream().map_err(std::io::Error::other));

  let stdout = match service.as_str() {
    "upload-pack" => {
      game_bucket
        .git
        .info_refs_upload(protocol, stream_reader)
        .await
    }
    "receive-pack" => {
      game_bucket
        .git
        .info_refs_receive(protocol, stream_reader)
        .await
    }
    _ => return Err(ResponseError::BadRequest("Invalid git service".to_owned())),
  };

  let stdout = match stdout {
    Ok(stdout) => stdout,
    Err(err) => {
      error!(error=?err, "failed to run git rpc");
      return Err(ResponseError::InternalServerError(
        "failed to run git rpc".to_owned(),
      ));
    }
  };

  let stdout_stream = ReaderStream::new(stdout);
  let header = tokio_stream::once(Ok(Bytes::from(format!(
    "{}0000",
    to_pkt_line(format!("# service=git-{service}\n"))
  ))));
  let stream = header.chain(stdout_stream);

  Ok((StatusCode::OK, headers, Body::from_stream(stream)))
}

async fn game_repo_git_rpc(
  service_name: &str, bucket: Bucket, game: game::Model, headers: HeaderMap, body: Body,
) -> Result<impl IntoResponse, ResponseError> {
  let expected_content_type = format!("application/x-git-{service_name}-request");
  let content_type = headers.get(CONTENT_TYPE).ok_or(ResponseError::BadRequest(
    "missing content type for git rpc".to_owned(),
  ))?;
  if content_type
    .to_str()
    .map_err(|_| ResponseError::BadRequest("invalid content type for git rpc".to_owned()))?
    != expected_content_type
  {
    return Err(ResponseError::BadRequest(
      "invalid content type for git rpc".to_owned(),
    ));
  }

  let protocol = get_protocol(&headers)?;
  let mut headers = HeaderMap::new();
  headers.insert(
    CONTENT_TYPE,
    format!("application/x-git-{service_name}-result")
      .parse()
      .unwrap(),
  );

  let game_bucket = bucket
    .at(
      game
        .bucket
        .clone()
        .ok_or(ResponseError::PreconditionFailed(
          "game bucket not found".to_owned(),
        ))?,
    )
    .await?;
  let stream_reader = StreamReader::new(body.into_data_stream().map_err(std::io::Error::other));

  let stdout = match service_name {
    "upload-pack" => game_bucket.git.upload_pack(protocol, stream_reader).await,
    "receive-pack" => game_bucket.git.receive_pack(protocol, stream_reader).await,
    _ => return Err(ResponseError::BadRequest("invalid git service".to_owned())),
  };

  let stdout = match stdout {
    Ok(stdout) => stdout,
    Err(err) => {
      error!(error=?err, "failed to run git rpc");
      return Err(ResponseError::InternalServerError(
        "failed to run git rpc".to_owned(),
      ));
    }
  };

  let stdout_stream = ReaderStream::new(stdout);

  Ok((StatusCode::OK, headers, Body::from_stream(stdout_stream)))
}

pub(super) async fn game_repo_git_receive_pack() -> Result<(), ResponseError> {
  Err(ResponseError::Gone(
    "this feature is not implemented".to_owned(),
  ))
}

pub(super) async fn game_repo_git_upload_pack(
  State(bucket): State<Bucket>, Extension(game): Extension<game::Model>, headers: HeaderMap,
  body: Body,
) -> Result<impl IntoResponse, ResponseError> {
  game_repo_git_rpc("upload-pack", bucket, game, headers, body).await
}
