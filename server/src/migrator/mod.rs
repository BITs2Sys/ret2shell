//! Used to init the new database and migrate the database from old versions.

use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::prelude::*;
use tracing::log::LevelFilter;

use crate::config::GlobalConfig;

mod m20210101_000001_create_institute;
mod m20210101_000002_create_user;
mod m20210101_000003_create_announcement;
mod m20210101_000004_create_game;
mod m20210101_000005_create_notification;
mod m20210101_000006_create_tag;
mod m20210101_000007_create_challenge;
mod m20210101_000008_create_team;
mod m20210101_000009_create_hint;
mod m20210101_000010_create_submission;
mod m20210101_000011_create_media;
mod m20210101_000012_create_ip_address;
mod m20210101_000013_create_writeup;
mod m20210101_000014_create_answer;
mod m20210101_000015_create_user_team_ref;
mod m20210101_000016_create_user_ip_ref;
mod m20210101_000017_create_instance;
mod m20210101_000018_create_action;
mod m20210101_000019_create_config;
mod m20210101_000020_create_calendar;
mod m20210101_000021_create_wiki;
mod m20210101_000022_create_plan;
mod m20210101_000023_create_plan_challenge_ref;
mod m20210101_000024_create_subscription;
mod m20210101_000025_create_subscription_user_ref;
mod m20210101_000026_create_extra;
mod m20210101_000027_create_cheat_record;
mod m20210101_000028_create_oauth;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20210101_000001_create_institute::Migration),
            Box::new(m20210101_000002_create_user::Migration),
            Box::new(m20210101_000003_create_announcement::Migration),
            Box::new(m20210101_000004_create_game::Migration),
            Box::new(m20210101_000005_create_notification::Migration),
            Box::new(m20210101_000006_create_tag::Migration),
            Box::new(m20210101_000007_create_challenge::Migration),
            Box::new(m20210101_000008_create_team::Migration),
            Box::new(m20210101_000009_create_hint::Migration),
            Box::new(m20210101_000010_create_submission::Migration),
            Box::new(m20210101_000011_create_media::Migration),
            Box::new(m20210101_000012_create_ip_address::Migration),
            Box::new(m20210101_000013_create_writeup::Migration),
            Box::new(m20210101_000014_create_answer::Migration),
            Box::new(m20210101_000015_create_user_team_ref::Migration),
            Box::new(m20210101_000016_create_user_ip_ref::Migration),
            Box::new(m20210101_000017_create_instance::Migration),
            Box::new(m20210101_000018_create_action::Migration),
            Box::new(m20210101_000019_create_config::Migration),
            Box::new(m20210101_000020_create_calendar::Migration),
            Box::new(m20210101_000021_create_wiki::Migration),
            Box::new(m20210101_000022_create_plan::Migration),
            Box::new(m20210101_000023_create_plan_challenge_ref::Migration),
            Box::new(m20210101_000024_create_subscription::Migration),
            Box::new(m20210101_000025_create_subscription_user_ref::Migration),
            Box::new(m20210101_000026_create_extra::Migration),
            Box::new(m20210101_000027_create_cheat_record::Migration),
        ]
    }
}

pub async fn initialize(config: &GlobalConfig) -> anyhow::Result<DatabaseConnection> {
    let mut connect_options = ConnectOptions::new(config.database.dsn());
    connect_options
        .acquire_timeout(std::time::Duration::from_secs(15))
        .sqlx_logging(true)
        .sqlx_logging_level(LevelFilter::Debug);

    let db: DatabaseConnection = Database::connect(connect_options).await?;
    Migrator::up(&db, None).await?;
    Ok(db)
}

pub async fn down(config: &GlobalConfig) -> anyhow::Result<()> {
    let mut connect_options = ConnectOptions::new(config.database.dsn());
    connect_options
        .sqlx_logging(true)
        .sqlx_logging_level(LevelFilter::Debug);

    let db: DatabaseConnection = Database::connect(connect_options).await?;
    Migrator::down(&db, None).await?;
    Ok(())
}
