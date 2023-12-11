use std::path::{Path, PathBuf};

use axum::extract::{multipart::MultipartError, Multipart};
use image::imageops::FilterType;
use mime_util::get_media_extension;
use nanoid::{alphabet, nanoid};
use ring::digest::{Context, SHA256};
use thiserror::Error;
use tokio::{
    fs::{create_dir_all, File},
    io::AsyncWriteExt,
};
use tracing::{debug, info, warn};

use crate::entity::media::Model as MediaModel;

mod mime_util;

#[derive(Debug, Error)]
pub enum MediaError {
    #[error("failed to extract file info from request")]
    ExtractError(#[from] MultipartError),
    #[error("failed to parse content type")]
    ParseContentTypeError(#[from] mime::FromStrError),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("image error: {0}")]
    ImageError(#[from] image::ImageError),
    #[error("unsupported file type: {0}")]
    UnsupportedFileType(String),
}

pub fn convert_hash_to_path(hash: &str) -> String {
    let mut path = String::from(&hash[..4]);
    path.insert(2, '/');
    path
}

pub fn convert_model_to_path(model: &MediaModel) -> String {
    let path = convert_hash_to_path(&model.hash);
    format!("{}/{}", path, &model.name)
}

pub async fn save_media(
    folder: &str, mut req: Multipart, temp_folder: &str, require_thumbnail: bool,
) -> Result<MediaModel, MediaError> {
    if let Some(mut file) = req.next_field().await? {
        let content_type = file.content_type().unwrap_or("unknown").to_string();
        let file_ext = get_media_extension(&content_type)?;
        let temp_id = nanoid!(21, &alphabet::SAFE);
        let temp_file_name = format!("{temp_id}.{file_ext}");
        create_dir_all(temp_folder).await?;
        let temp_file_path = format!("{temp_folder}/{temp_file_name}");
        let mut temp_file = File::create(&temp_file_path).await?;
        let mut context = Context::new(&SHA256);
        while let Some(chunk) = file.chunk().await? {
            context.update(&chunk);
            temp_file.write_all(&chunk).await?;
        }
        let hash = hex::encode(context.finish().as_ref());
        let file_name = format!("{hash}.{file_ext}");
        let model = MediaModel {
            name: file_name,
            hash,
            ..Default::default()
        };
        create_dir_all(format!(
            "{folder}/{path}",
            path = convert_hash_to_path(&model.hash)
        ))
        .await?;
        create_dir_all(format!(
            "{folder}/thumbnails/{path}",
            folder = folder,
            path = convert_hash_to_path(&model.hash)
        ))
        .await?;
        let model_path = convert_model_to_path(&model);
        let dest_path = format!("{folder}/{model_path}");
        tokio::fs::rename(&temp_file_path, &dest_path).await?;
        if require_thumbnail {
            let thumb_dest_path = format!("{folder}/thumbnails/{model_path}");
            make_thumbnail(&dest_path, &thumb_dest_path, 256).await?;
        }
        Ok(model)
    } else {
        Err(MediaError::UnsupportedFileType("empty".to_string()))
    }
}

pub async fn get_media(folder: &str, path: &str) -> Result<File, MediaError> {
    let base_path = PathBuf::from(folder).canonicalize()?;
    let path = base_path.join(path).canonicalize()?;
    if !path.starts_with(base_path) {
        return Err(MediaError::IOError(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "permission denied",
        )));
    }
    let file = File::open(&path).await?;
    Ok(file)
}

async fn make_thumbnail<PA, PB>(
    original: PA, dest: PB, longest_edge: u32,
) -> Result<(), MediaError>
where
    PA: AsRef<Path>,
    PB: AsRef<Path>, {
    // prevent generate thumbnail repeatedly
    if tokio::fs::metadata(&dest).await.is_ok() {
        return Ok(());
    }
    // prevent generate thumbnail for svg
    if original.as_ref().extension().unwrap_or_default() == "svg" {
        let _ = tokio::fs::hard_link(original, dest).await;
        return Ok(());
    }
    debug!("generating thumbnail for {}", original.as_ref().display());
    let img = image::open(&original)?;

    match img
        .resize(longest_edge, longest_edge, FilterType::Nearest)
        .save(&dest)
    {
        Err(err) => {
            warn!("resize image to thumbnail error: {err}");
            info!("image will be directly save to {}", dest.as_ref().display());
            let _ = tokio::fs::hard_link(original, dest).await;
            Ok(())
        }
        Ok(_) => Ok(()),
    }
}
