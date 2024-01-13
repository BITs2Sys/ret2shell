//! Bucket module

// TODO: should implement dynamic attachments feature in right way.

use std::path::PathBuf;

use axum::extract::{multipart::MultipartError, Multipart};
use redb::{Database, Error as RedbError, ReadableTable, TableDefinition};
use ring::digest::{Context, SHA256};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::{config::GlobalConfig, entity::challenge, utility::string::deunicode_str};

const FILES_HASH_TABLE: TableDefinition<&str, &str> = TableDefinition::new("files");

#[derive(Debug, Error)]
pub enum BucketError {
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Bucket directory not exist")]
    BucketDirNotExist,
    #[error("Bucket is not initialized")]
    BucketNotInitialized,
    #[error("failed to extract file info from request")]
    ExtractError(#[from] MultipartError),
    #[error("serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("file not found")]
    FileNotFound,
    #[error("redb error: {0}")]
    RedbError(#[from] RedbError),
    // #[error("particial files upload failed")]
    // ParticialUploadFailed,
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
        // tokio::fs::create_dir_all(&bucket_path.join("dynamic")).await?;
        Database::create(bucket_path.join("files.db")).map_err(RedbError::from)?;
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

impl PartialOrd for AttachmentMeta {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for AttachmentMeta {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for AttachmentMeta {}

impl Ord for AttachmentMeta {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.hash.cmp(&other.hash)
    }
}

macro_rules! check_bucket_db {
    ($t:literal, $config:expr, $challenge:expr) => {{
        let bucket_path: PathBuf = $config.bucket.path.clone().into();
        if !bucket_path.exists() {
            return Err(BucketError::BucketDirNotExist);
        }
        let Some(challenge_folder) = $challenge.bucket.clone() else {
            return Err(BucketError::BucketNotInitialized);
        };
        let db_path = bucket_path.join(&challenge_folder).join("files.db");
        let db = Database::open(&db_path).map_err(RedbError::from)?;
        let bucket_path = bucket_path.join(&challenge_folder).join($t);
        if !bucket_path.exists() {
            return Err(BucketError::BucketNotInitialized);
        }
        (db, bucket_path)
    }};
}

pub async fn upload_static_attachment(
    config: &GlobalConfig, challenge: &challenge::Model, mut req: Multipart,
) -> Result<(), BucketError> {
    let (db, bucket_path) = check_bucket_db!("static", config, challenge);

    while let Some(mut will_send) = req.next_field().await? {
        let temp_name = nanoid::nanoid!(21, &nanoid::alphabet::SAFE);
        let filename = will_send
            .file_name()
            .map(str::to_string)
            .unwrap_or(temp_name.clone());
        let filename = deunicode_str(&filename);
        let file_path = bucket_path.join(&filename);
        let mut file = File::create(&file_path).await?;
        let mut context = Context::new(&SHA256);
        while let Some(chunk) = will_send.chunk().await? {
            context.update(&chunk);
            file.write_all(&chunk).await?;
        }
        let hash = hex::encode(context.finish().as_ref());
        // tokio::fs::rename(&bucket_path.join(&temp_name),
        // &bucket_path.join(&hash)).await?;
        let meta = AttachmentMeta {
            name: filename.clone(),
            hash: hash.clone(),
        };
        let write_txn = db.begin_write().map_err(RedbError::from)?;
        {
            let mut table = write_txn
                .open_table(FILES_HASH_TABLE)
                .map_err(RedbError::from)?;
            table
                .insert(&*hash, &*format!("static/{hash}"))
                .map_err(RedbError::from)?;
        }
        write_txn.commit().map_err(RedbError::from)?;
        tokio::fs::write(format!("{filename}.meta"), serde_json::to_string(&meta)?).await?;
    }

    Ok(())
}

pub async fn get_static_attachment_list(
    config: &GlobalConfig, challenge: &challenge::Model,
) -> Result<Vec<AttachmentMeta>, BucketError> {
    let (_, bucket_path) = check_bucket_db!("static", config, challenge);

    let mut result = Vec::new();
    while let Some(entry) = tokio::fs::read_dir(&bucket_path)
        .await?
        .next_entry()
        .await?
    {
        if entry.path().extension().unwrap_or_default() == "meta" {
            let meta: AttachmentMeta = serde_json::from_str(
                &tokio::fs::read_to_string(entry.path().to_str().unwrap_or_default()).await?,
            )?;
            result.push(meta);
        }
    }

    Ok(result)
}

pub async fn get_file_by_hash(
    config: &GlobalConfig, challenge: &challenge::Model, hash: &str,
) -> Result<(AttachmentMeta, File), BucketError> {
    let (db, bucket_path) = check_bucket_db!("", config, challenge);

    let read_txn = db.begin_read().map_err(RedbError::from)?;
    let table = read_txn
        .open_table(FILES_HASH_TABLE)
        .map_err(RedbError::from)?;
    let path = table
        .get(hash)
        .map_err(RedbError::from)?
        .ok_or(BucketError::FileNotFound)?;
    let full_path = bucket_path.join(path.value());
    let meta: AttachmentMeta = serde_json::from_str(
        &tokio::fs::read_to_string(
            full_path
                .with_extension("meta")
                .to_str()
                .unwrap_or_default(),
        )
        .await?,
    )?;
    if !full_path.exists() {
        let write_txn = db.begin_write().map_err(RedbError::from)?;
        {
            let mut table = write_txn
                .open_table(FILES_HASH_TABLE)
                .map_err(RedbError::from)?;
            table.remove(hash).map_err(RedbError::from)?;
        }
        write_txn.commit().map_err(RedbError::from)?;
        return Err(BucketError::FileNotFound);
    }
    let file = File::open(full_path).await?;

    Ok((meta, file))
}

pub async fn delete_file_by_hash(
    config: &GlobalConfig, challenge: &challenge::Model, hash: &str,
) -> Result<(), BucketError> {
    let (db, bucket_path) = check_bucket_db!("", config, challenge);

    let rel_path;

    let write_txn = db.begin_write().map_err(RedbError::from)?;
    {
        let mut table = write_txn
            .open_table(FILES_HASH_TABLE)
            .map_err(RedbError::from)?;
        rel_path = table
            .get(hash)
            .map_err(RedbError::from)?
            .ok_or(BucketError::FileNotFound)?
            .value()
            .to_owned();
        table.remove(hash).map_err(RedbError::from)?;
    }
    write_txn.commit().map_err(RedbError::from)?;

    let full_path = bucket_path.join(rel_path);
    tokio::fs::remove_file(&full_path).await?;
    tokio::fs::remove_file(&full_path.with_extension("meta")).await?;

    Ok(())
}
