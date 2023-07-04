//! Used to init the new database and migrate the database from old versions.
//!
//!

use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::prelude::*;
use tracing::log::LevelFilter;

use crate::config::GlobalConfig;

mod m20221109_000001_create_announcement;
mod m20221109_000002_create_institute;
mod m20221109_000003_create_game;
mod m20221109_000004_create_notification;
mod m20221109_000005_create_user;
mod m20221109_000006_create_tag;
mod m20221109_000007_create_challenge;
mod m20221109_000008_create_submission;
mod m20221109_000009_create_hint;
mod m20221110_000001_create_team;
mod m20221110_000002_create_media;
mod m20221110_000003_create_ip_address;
mod m20221110_000004_create_writeup;
mod m20221110_000005_create_answer;
mod m20221110_000006_create_user_team_ref;
mod m20221110_000007_create_user_ip_ref;
mod m20230430_000001_create_instance;
mod m20230502_000001_create_action;
mod m20230704_000001_create_platform_info;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20221109_000001_create_announcement::Migration),
            Box::new(m20221109_000002_create_institute::Migration),
            Box::new(m20221109_000003_create_game::Migration),
            Box::new(m20221109_000004_create_notification::Migration),
            Box::new(m20221109_000005_create_user::Migration),
            Box::new(m20221109_000006_create_tag::Migration),
            Box::new(m20221109_000007_create_challenge::Migration),
            Box::new(m20221109_000008_create_submission::Migration),
            Box::new(m20221109_000009_create_hint::Migration),
            Box::new(m20221110_000001_create_team::Migration),
            Box::new(m20221110_000002_create_media::Migration),
            Box::new(m20221110_000003_create_ip_address::Migration),
            Box::new(m20221110_000004_create_writeup::Migration),
            Box::new(m20221110_000005_create_answer::Migration),
            Box::new(m20221110_000006_create_user_team_ref::Migration),
            Box::new(m20221110_000007_create_user_ip_ref::Migration),
            Box::new(m20230430_000001_create_instance::Migration),
            Box::new(m20230502_000001_create_action::Migration),
            Box::new(m20230704_000001_create_platform_info::Migration),
        ]
    }
}

pub async fn initialize(config: &GlobalConfig) -> anyhow::Result<DatabaseConnection> {
    let mut connect_options = ConnectOptions::new(config.database.dsn());
    connect_options
        .sqlx_logging(true)
        .sqlx_logging_level(LevelFilter::Debug);

    let db: DatabaseConnection = Database::connect(connect_options).await?;
    Migrator::up(&db, None).await?;
    Ok(db)
}

#[allow(dead_code)]
pub async fn down(config: &GlobalConfig) -> anyhow::Result<()> {
    let mut connect_options = ConnectOptions::new(config.database.dsn());
    connect_options
        .sqlx_logging(true)
        .sqlx_logging_level(LevelFilter::Debug);

    let db: DatabaseConnection = Database::connect(connect_options).await?;
    let _ = Migrator::down(&db, None).await;
    Ok(())
}
