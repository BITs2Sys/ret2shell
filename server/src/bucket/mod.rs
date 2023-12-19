use std::path::PathBuf;

use crate::{config::GlobalConfig, entity::challenge, utility::string::deunicode_str};
use axum::extract::{multipart::MultipartError, Multipart};
use ring::digest::{Context, SHA256};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{fs::File, io::AsyncWriteExt};

#[derive(Debug, Error)]
pub enum BucketError {
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Bucket directory not exist")]
    BucketDirNotExist,
    #[error("Bucket is not initialized")]
    BucketNotInitialized,
    #[error("file does not have a name")]
    NoFileName,
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
        tokio::fs::create_dir_all(&bucket_path.join("static")).await?;
        tokio::fs::create_dir_all(&bucket_path.join("dynamic")).await?;
    }
    Ok(challenge::Model {
        bucket: Some(bucket_name),
        ..challenge.clone()
    })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttachmentMeta {
    pub name: String,
    pub hash: String,
}

pub async fn upload_static_attachment(
    config: &GlobalConfig, challenge: &challenge::Model, mut req: Multipart,
) -> Result<(), BucketError> {
    let bucket_path: PathBuf = config.bucket.path.clone().into();
    if !bucket_path.exists() {
        return Err(BucketError::BucketDirNotExist);
    }
    let Some(challenge_folder) = challenge.bucket.clone() else {
        return Err(BucketError::BucketNotInitialized);
    };
    let bucket_path = bucket_path.join(&challenge_folder).join("static");
    if !bucket_path.exists() {
        return Err(BucketError::BucketNotInitialized);
    }

    while let Some(mut will_send) = req.next_field().await? {
        let file_name = nanoid::nanoid!(21, &nanoid::alphabet::SAFE);
        let real_name = will_send
            .file_name()
            .map(str::to_string)
            .unwrap_or(file_name.clone());
        let mut file = File::create(&bucket_path.join(&file_name)).await?;
        let mut context = Context::new(&SHA256);
        while let Some(chunk) = will_send.chunk().await? {
            context.update(&chunk);
            file.write_all(&chunk).await?;
        }
        let hash = hex::encode(context.finish().as_ref());
        let meta = AttachmentMeta {
            name: real_name.clone(),
            hash,
        };
        tokio::fs::write(format!("{file_name}.meta"), serde_json::to_string(&meta)?).await?;
    }

    Ok(())
}

pub async fn upload_dynamic_attachment(
    config: &GlobalConfig, challenge: &challenge::Model, mut req: Multipart,
) -> Result<(), BucketError> {
    let bucket_path: PathBuf = config.bucket.path.clone().into();
    if !bucket_path.exists() {
        return Err(BucketError::BucketDirNotExist);
    }
    let Some(challenge_folder) = challenge.bucket.clone() else {
        return Err(BucketError::BucketNotInitialized);
    };
    let bucket_path = bucket_path.join(&challenge_folder).join("dynamic");
    if !bucket_path.exists() {
        return Err(BucketError::BucketNotInitialized);
    }

    while let Some(mut will_send) = req.next_field().await? {
        let file_name = nanoid::nanoid!(21, &nanoid::alphabet::SAFE);
        let real_name = will_send
            .file_name()
            .map(str::to_string)
            .unwrap_or(file_name.clone());
        let mut file = File::create(&bucket_path.join(&file_name)).await?;
        let mut context = Context::new(&SHA256);
        while let Some(chunk) = will_send.chunk().await? {
            context.update(&chunk);
            file.write_all(&chunk).await?;
        }
        let hash = hex::encode(context.finish().as_ref());
        let meta = AttachmentMeta {
            name: real_name.clone(),
            hash,
        };
        tokio::fs::write(format!("{file_name}.meta"), serde_json::to_string(&meta)?).await?;
    }

    Ok(())
}
