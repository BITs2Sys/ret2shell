//! # Intro
//!
//! `ret2shell`: a feature-riches CTF challenge platform. This website contains
//! the implementation details of `ret2shell`, not the user manual.
//!
//! This documentation is generated from the source code of `ret2shell`.
//! If you want to contribute to this project, refer this.
//!
//! > **DO NOT USE THIS PROJECT AS A LIBRARY IN YOUR PROJECT.
//! ALL THE MEMBERS ARE PRIVATE AND WILL CHANGE WITHOUT NOTICE.**

#![warn(missing_docs)]
#![forbid(unsafe_code)]

use std::{io::Write, net::SocketAddr};

use clap::{command, Parser, Subcommand};
use colored::Colorize;
use config::GlobalConfig;
use rustc_version::version;
// use tokio::signal;
use tracing::{error, info, warn};

use crate::controller::GlobalState;

mod audit;
mod bucket;
mod cache;
mod captcha;
mod config;
mod controller;
mod email;
mod entity;
mod license;
mod logging;
mod media;
mod migrator;
mod oauth;
mod queue;
mod service;
mod traffic;
mod utility;
mod workflow;

/// Clap arg definition.
#[derive(Parser, Debug)]
#[command(
    author = "Reverier-Xu <reverier.xu@woooo.tech>",
    version,
    about = "Ret 2 Shell Challenge API Platform",
    long_about = r#"
Ret 2 Shell Challenge API Platform

THE CONTENTS OF THIS PROJECT ARE PROPRIETARY AND CONFIDENTIAL.
UNAUTHORIZED COPYING, TRANSFERRING OR REPRODUCTION OF THE CONTENTS OF THIS PROJECT,
VIA ANY MEDIUM IS STRICTLY PROHIBITED.

If you have any problems, please contact tech support <ret2shell@woooo.tech>.
    "#
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
    /// Remove all data and drop database, NEVER USE IT AT PRODUCTION
    /// ENVIRONMENT.
    Erase,
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
        Some(Commands::Erase) => erase(config).await,
        None => up(config).await,
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
    println!(
        "[START UP] {} {}",
        "Ret 2 Shell".bold(),
        format!(
            "{}-{}-{}",
            env!("CARGO_PKG_VERSION"),
            git_version::git_version!(
                args = ["--abbrev=8", "--always", "--dirty=*"],
                fallback = "unknown"
            )
            .to_uppercase(),
            version().unwrap().to_string()
        )
        .dimmed()
    );
    println!(
        "----------------------------- {} -----------------------------",
        "server log starts here".to_uppercase().bold()
    );
}

/// Power on the server.
async fn up(config: GlobalConfig) -> anyhow::Result<()> {
    greet();

    warn!(">> Server initialization started <<");
    // debug!("LOGGER TEST: DEBUG MESSAGE");

    let license = match license::check_license() {
        Ok(license) => license,
        Err(err) => {
            error!("License check failed: {}", err.to_string().red());
            error!("Please contact tech support <ret2shell@woooo.tech>.");
            return Err(err.into());
        }
    };

    info!(
        "Licensed to {} ({}), will expire at {}",
        license.issuer, license.website, license.date
    );

    info!("Loading module: < Audit >");
    let auditor = audit::initialize(&config).await?;
    info!("Loading module: < Database >");
    let db = migrator::initialize(&config).await?;
    info!("Loading module: < Cache >");
    let cache = cache::initialize(&config).await?;
    info!("Loading module: < Message Queue >");
    let queue = queue::initialize(&config).await?;
    info!("Loading module: < Kubernetes >");
    let cluster = service::initialize(&config).await?;
    info!("Loading module: < Email >");
    email::initialize(&config, &db, &queue).await?;
    let state = GlobalState {
        config: config.clone(),
        auditor,
        db,
        cache,
        queue,
        cluster,
        license,
    };
    let router = controller::initialize(&config, state).await?;
    info!("Router constructed.");

    warn!(">> Server initialization finished <<");

    info!("Starting server...");

    let addr_str = format!("{}:{}", &config.server.host, &config.server.port);

    let addr = tokio::net::TcpListener::bind(addr_str.clone())
        .await
        .expect("Failed to bind server address");
    info!("Server started at [ {} ]", addr_str);
    axum::serve(
        addr,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Failed to start server.");

    Ok(())
}

/// Remove all data and drop database, please use it carefully.
///
/// > NEVER USE IT AT PRODUCTION ENVIRONMENT.
async fn erase(config: GlobalConfig) -> anyhow::Result<()> {
    greet();
    println!(
        "{}",
        "WARNING: this operation will drop all your data!"
            .bold()
            .bright_red()
    );
    println!(
        "{}",
        "Please only run it on development server."
            .bold()
            .bright_red()
    );
    print!(
        "Are you sure to continue? [{}/{}]: ",
        "yes".red(),
        "NO".bold().green()
    );
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if !input.trim().to_lowercase().eq("yes") {
        warn!("Cleanup aborted");
        return Ok(());
    }
    warn!(">> Server cleanup started <<");
    migrator::down(&config).await?;
    info!("Cleanup done: < Database >");
    warn!(">> Server cleanup finished <<");
    Ok(())
}

// async fn graceful_shutdown() {
//     let ctrl_c = async {
//         signal::ctrl_c()
//             .await
//             .expect("Failed to install SIGTERM handler")
//     };

//     #[cfg(unix)]
//     let terminate = async {
//         signal::unix::signal(signal::unix::SignalKind::terminate())
//             .expect("failed to install SIGTERM handler")
//             .recv()
//             .await;
//     };

//     #[cfg(not(unix))]
//     let terminate = std::future::pending::<()>();

//     tokio::select! {
//         _ = ctrl_c => {},
//         _ = terminate => {},
//     }

//     info!("Server is shutting down...");
//     // TODO: add `pusher module`(websocket) graceful shutdown handle code
// here.     info!("Server shutdown completed, bye bye.");
//     exit(0);
// }
