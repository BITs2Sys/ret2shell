use std::path::Path;

use axum::{
    body::Body,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use tokio::fs;
use tokio_util::io::ReaderStream;

use crate::traits::ResponseError;

pub async fn send_file(path: impl AsRef<Path>) -> Result<Response, ResponseError> {
    let file = fs::File::open(path.as_ref()).await?;
    let mut header = HeaderMap::new();
    header.insert("Content-Length", file.metadata().await?.len().into());
    let stream = ReaderStream::new(file);
    Ok((StatusCode::OK, header, Body::from_stream(stream)).into_response())
}
