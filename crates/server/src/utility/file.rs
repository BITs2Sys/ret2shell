use std::path::Path;

use axum::{
    body::Body,
    response::{Response},
};
use tokio::fs;
use tokio_util::io::ReaderStream;

use crate::traits::ResponseError;

pub async fn send_file(path: impl AsRef<Path>) -> Result<Response, ResponseError> {
    let file = fs::File::open(path.as_ref()).await?;
    let stream = ReaderStream::new(file);
    Ok(Response::new(Body::from_stream(stream)))
}
