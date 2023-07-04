//! # Intro
//!
//! `ret2shell`: a feature-riched CTF challenge platform. This website contains the
//! implemention details of `ret2shell`, not the user manual.
//!
//! This documentation is generated from the source code of `ret2shell`.
//! If you want to contribute to this project, refer this.
//!
//! > **DO NOT USE THIS PROJECT AS A LIBRARY IN YOUR PROJECT.
//! ALL THE MEMBERS ARE PRIVATE AND WILL CHANGE WITHOUT NOTICE.**
//!

#![warn(missing_docs)]

use clap::{command, Parser, Subcommand};
use config::GlobalConfig;
use tracing::{error, info, warn};

mod audit;
mod bucket;
mod cache;
mod captcha;
mod checker;
mod config;
mod controller;
mod entity;
mod logging;
mod media;
mod migrator;
mod oauth;
mod queue;
mod service;
mod traffic;
mod utility;

/// Clap arg definition.
#[derive(Parser, Debug)]
#[command(
    author = "Reverier-Xu <reverier.xu@woooo.tech>",
    version,
    about = "Ret 2 Shell Challenge API Platform",
    long_about = "Ret 2 Shell Challenge API Platform\nIf you have any problems, please contact tech support <ret2shell@woooo.tech>"
)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Clap subcommands.
#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the server.
    Up,
    /// Remove all data and drop database, NEVER USE IT AT PRODUCTION ENVIRONMENT.
    Down,
}

/// Server entry.
#[tokio::main]
async fn main() {
    let config = GlobalConfig::load().expect("Failed to load configuration");
    let (_console_guard, _file_guard) = logging::initialize(&config)
        .await
        .expect("Failed to initialize logger");
    // Parse command line arguments
    let args: Args = Args::parse();
    match match args.command {
        Some(Commands::Up) => up(config).await,
        Some(Commands::Down) => down(config).await,
        None => {
            warn!("No command specified, default to `up`");
            up(config).await
        }
    } {
        Ok(_) => {}
        Err(err) => {
            error!("Serve execution failed with error: {}", err);
            error!("If you ensure that this is a bug, please report it to <ret2shell@woooo.tech> for tech support.");
        }
    }
}

/// Show greet information.
fn greet() {
    info!("-*- Welcome to Ret 2 Shell! -*-");
    info!("\"怀有希望的人和满天的星星一样 是永远都不会孤独的\"")
}

/// Power on the server.
async fn up(config: GlobalConfig) -> anyhow::Result<()> {
    greet();
    warn!(">> Server initialization started <<");
    let _auditor = audit::initialize(&config).await?;
    info!("Loaded Module: < Audit >");
    let _dbconn = migrator::initialize(&config).await?;
    info!("Loaded Module: < Database >");

    warn!(">> Server initialization finished <<");
    Ok(())
}

/// Remove all data and drop database, please use it carefully.
///
/// > NEVER USE IT AT PRODUCTION ENVIRONMENT.
async fn down(_config: GlobalConfig) -> anyhow::Result<()> {
    greet();
    Ok(())
}
