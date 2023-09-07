use sea_orm::{entity::prelude::*, ActiveValue};
use serde::{Deserialize, Serialize};

use crate::captcha::Validator;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "platform_info")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(column_type = "JsonBinary")]
    pub platform: Option<Json>,
    #[sea_orm(column_type = "JsonBinary")]
    pub auth: Option<Json>,
    #[sea_orm(column_type = "JsonBinary")]
    pub captcha: Option<Json>,
    #[sea_orm(column_type = "JsonBinary")]
    pub email: Option<Json>,
    #[sea_orm(column_type = "JsonBinary")]
    pub media: Option<Json>,
    #[sea_orm(column_type = "JsonBinary")]
    pub pusher: Option<Json>,
}

macro_rules! from_json_value {
    ($name:expr) => {
        serde_json::from_value($name.expect("failed to deserialize platform info"))
            .expect("failed to deserialize platform info")
    };
}

macro_rules! to_json_value {
    ($name:expr) => {
        Some(serde_json::to_value($name).expect("failed to serialize platform info"))
    };
}

impl From<PlatformInfoModel> for Model {
    fn from(platform_info_model: PlatformInfoModel) -> Self {
        Self {
            id: 0,
            auth: to_json_value!(platform_info_model.auth),
            platform: to_json_value!(platform_info_model.platform),
            captcha: to_json_value!(platform_info_model.captcha),
            email: to_json_value!(platform_info_model.email),
            media: to_json_value!(platform_info_model.media),
            pusher: to_json_value!(platform_info_model.pusher),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlatformInfoModel {
    pub auth: Option<Auth>,
    pub platform: Option<Platform>,
    pub captcha: Option<Captcha>,
    pub email: Option<Email>,
    pub media: Option<Media>,
    pub pusher: Option<Pusher>,
}

impl From<Model> for PlatformInfoModel {
    fn from(platform_info: Model) -> Self {
        Self {
            auth: from_json_value!(platform_info.auth),
            platform: from_json_value!(platform_info.platform),
            captcha: from_json_value!(platform_info.captcha),
            email: from_json_value!(platform_info.email),
            media: from_json_value!(platform_info.media),
            pusher: from_json_value!(platform_info.pusher),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Platform {
    pub name: String,
    pub footer_info: String,
    pub footer_url: String,
    pub subject_info: String,
    pub subject_url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Captcha {
    /// Whether captcha functionality is enabled or not.
    pub enabled: bool,
    /// The captcha difficulty.
    pub difficulty: Option<u16>,
    /// The captcha validator to use.
    pub validator: Validator,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Auth {
    pub signing_key: String,
    pub buffer_time: i64,
    pub expires_time: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Email {
    /// Whether email functionality is enabled or not.
    pub enabled: bool,
    /// The email server host.
    pub host: String,
    /// The email server port.
    pub port: u16,
    /// The email address used as the sender.
    pub sender: String,
    /// The username for authentication with the email server.
    pub username: String,
    /// The password for authentication with the email server.
    pub password: String,
    /// The TLS configuration for secure email communication.
    pub tls: String,
    /// The email body for reset password emails, could be 'none' | 'tls' | 'starttls'
    pub reset_password_email_body: String,
    /// The email subject for reset password emails.
    pub reset_password_email_subject: String,
    /// The email body for email verification emails.
    pub verify_email_body: String,
    /// The email subject for email verification emails.
    pub verify_email_subject: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Media {
    /// `anti_theft` is a flag to enable or disable anti-theft protection for media files.
    pub anti_theft: bool,
    /// `limit` is the maximum allowed size (in bytes) for media files.
    pub limit: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pusher {
    /// Indicates whether the Pusher service is enabled or not.
    pub enabled: bool,
    /// The authentication token for the Pusher service.
    pub token: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub async fn get_platform_info(
    db: &DatabaseConnection,
) -> Result<Option<PlatformInfoModel>, DbErr> {
    let platform_info = Entity::find().one(db).await?;
    match platform_info {
        Some(platform_info) => Ok(Some(platform_info.into())),
        None => Ok(None),
    }
}

pub async fn update_platform_info(
    db: &DatabaseConnection,
    platform_info: PlatformInfoModel,
) -> Result<(), DbErr> {
    let original_info = Entity::find().one(db).await?;
    match original_info {
        Some(original_info) => {
            let platform_info: Model = platform_info.into();
            let mut platform_info: ActiveModel = platform_info.into();
            platform_info = platform_info.reset_all();
            platform_info.id = ActiveValue::Unchanged(original_info.id);
            platform_info.update(db).await?;
        }
        None => {
            let platform_info: Model = platform_info.into();
            let platform_info: ActiveModel = platform_info.into();
            platform_info.insert(db).await?;
        }
    }
    Ok(())
}
