//! Bucket module

use std::path::PathBuf;

use axum::extract::multipart::MultipartError;
use thiserror::Error;

use crate::{config::GlobalConfig, entity::challenge, utility::string::deunicode_str};

#[derive(Debug, Error)]
pub enum BucketError {
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Bucket directory not exist")]
    BucketDirNotExist,
    #[error("failed to extract file info from request")]
    ExtractError(#[from] MultipartError),
    #[error("serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

fn generate_bucket_name_for_challenge(challenge: &challenge::Model) -> String {
    format!(
        "{}_{}_{}",
        challenge.game_id,
        challenge.id,
        deunicode_str(&challenge.name)
    )
}

pub async fn init_challenge_bucket(
    config: &GlobalConfig, challenge: &challenge::Model,
) -> Result<challenge::Model, BucketError> {
    let bucket_name = generate_bucket_name_for_challenge(challenge);
    let bucket_path: PathBuf = config.bucket.path.clone().into();
    if !bucket_path.exists() {
        return Err(BucketError::BucketDirNotExist);
    }
    let bucket_path = bucket_path.join(bucket_name.clone());
    if !bucket_path.exists() {
        tokio::fs::create_dir_all(&bucket_path).await?;
    }
    Ok(challenge::Model {
        bucket: Some(bucket_name),
        ..challenge.clone()
    })
}
