use async_nats::jetstream::{
    consumer::{pull::Config, Consumer, StreamError},
    context::PublishErrorKind,
    Context,
};
use lettre::{
    message::{header, SinglePart},
    transport::smtp::{
        authentication::Credentials,
        client::{Tls, TlsParameters},
    },
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio_stream::StreamExt;
use tracing::{debug, error, info};

use crate::{config::GlobalConfig, entity::config::get_config};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Email {
    pub name: String,
    pub email: String,
    pub subject: String,
    pub content: String,
}

/// Constructs an email message using the given `Email` struct and
/// `CybertermConfig`.
///
/// # Arguments
///
/// * `email` - A reference to an `Email` struct containing the email details.
/// * `config` - A reference to a `CybertermConfig` containing the email
///   configuration.
///
/// # Returns
///
/// * A `Message` containing the constructed email.
async fn construct_email(
    email: &Email, sender: impl AsRef<str>, username: impl AsRef<str>,
) -> Message {
    Message::builder()
        .from(
            format!("{} <{}>", sender.as_ref(), username.as_ref())
                .parse()
                .unwrap(),
        )
        .to(format!("{} <{}>", email.name, email.email).parse().unwrap())
        .subject(&email.subject)
        .singlepart(
            SinglePart::builder()
                .header(header::ContentType::TEXT_HTML)
                .body(String::from(&email.content)),
        )
        .unwrap()
}

pub async fn initialize(
    _config: &GlobalConfig, db: &DatabaseConnection, queue: &Context,
) -> anyhow::Result<()> {
    let stream = queue
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: "email".to_string(),
            max_messages: 10_000,
            ..Default::default()
        })
        .await?;

    let subscriber = stream
        .get_or_create_consumer(
            "email",
            async_nats::jetstream::consumer::pull::Config {
                durable_name: Some("email".to_string()),
                ..Default::default()
            },
        )
        .await?;
    let db = db.clone();
    let future = email_worker(subscriber, db);
    tokio::spawn(future);
    Ok(())
}

#[derive(Error, Debug)]
pub enum EmailError {
    #[error("Invalid email tls configuration: {0}")]
    InvalidEmailTlsConfiguration(String),
    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::error::DbErr),
    #[error("mailer error: {0}")]
    MailerError(#[from] lettre::transport::smtp::Error),
    #[error("stream error: {0}")]
    StreamError(#[from] StreamError),
    #[error("serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("nats error: {0}")]
    NatsError(#[from] async_nats::error::Error<PublishErrorKind>),
}

async fn email_worker(
    subscriber: Consumer<Config>, db_conn: DatabaseConnection,
) -> anyhow::Result<()> {
    let mut messages = subscriber
        .stream()
        .max_messages_per_batch(10)
        .messages()
        .await?;
    while let Some(message) = messages.next().await {
        if let Ok(message) = message {
            let email = String::from_utf8(message.message.payload.to_vec());
            if let Ok(email) = email {
                if let Ok(email) = serde_json::from_str::<Email>(&email) {
                    if let Err(err) = send_email_impl(email.clone(), &db_conn).await {
                        error!("Failed to send email: {:?}", err);
                    } else {
                        info!("Successfully sent email: {:?}", email);
                        message.ack().await.ok();
                    }
                } else {
                    error!("Failed to deserialize email: {:?}", email);
                }
            } else {
                error!("Failed to convert email message to string: {:?}", email);
            }
        } else {
            error!("Failed to receive message from nats: {:?}", message);
        }
    }
    Ok(())
}

async fn send_email_impl(email: Email, db_conn: &DatabaseConnection) -> Result<(), EmailError> {
    let hot_config = get_config(db_conn).await?;
    let smtp_credentials = Credentials::new(
        hot_config.email.username.clone(),
        hot_config.email.password.clone(),
    );
    debug!(
        "smtp_credentials: {} {}",
        hot_config.email.username, hot_config.email.password
    );
    debug!(
        "smtp host: {} {}:{}",
        hot_config.email.tls, hot_config.email.host, hot_config.email.port
    );
    debug!("email: {:?}", email);
    let mailer: AsyncSmtpTransport<Tokio1Executor> = match hot_config.email.tls.as_str() {
        "starttls" => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&hot_config.email.host),
        "tls" => Ok(
            AsyncSmtpTransport::<Tokio1Executor>::relay(&hot_config.email.host)?.tls(Tls::Wrapper(
                TlsParameters::builder(hot_config.email.host.clone())
                    .build()
                    .unwrap(),
            )),
        ),
        "none" => Ok(AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(
            &hot_config.email.host,
        )),
        _ => {
            return Err(EmailError::InvalidEmailTlsConfiguration(
                hot_config.email.tls.clone(),
            ))
        }
    }?
    .port(hot_config.email.port)
    .credentials(smtp_credentials)
    .build();
    let email = construct_email(&email, &hot_config.email.sender, &hot_config.email.username).await;
    mailer.send(email).await?;
    Ok(())
}

pub async fn send_email(
    email: &Email, queue: &async_nats::jetstream::Context,
) -> Result<(), EmailError> {
    queue
        .publish("email".to_owned(), serde_json::to_vec(email)?.into())
        .await?;
    Ok(())
}
