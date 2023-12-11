//! This module setup the logger of `tracing`.
//!
//! In the initialization stage, the log writer will be guarded by their file
//! descriptor, the lifetime of the file descriptor is the same as the log
//! writer. You should keep the file descriptor until application exit.

use std::path::Path;

use tracing_appender::{non_blocking, non_blocking::WorkerGuard, rolling};
use tracing_subscriber::{fmt::Layer, prelude::*};

use crate::config::GlobalConfig;

/// Initialize the logger.
pub async fn initialize(config: &GlobalConfig) -> anyhow::Result<(WorkerGuard, WorkerGuard)> {
    tokio::fs::create_dir_all(&config.logging.directory).await?;
    let file_appender = rolling::daily(
        Path::new(&config.logging.directory).canonicalize()?,
        "ret2shell.log",
    );
    // println!("log config: {:?}", config.logging);

    let (non_blocking_file, _file_guard) = non_blocking(file_appender);
    let (non_blocking_console, _console_guard) = non_blocking(std::io::stdout());
    let file_log_layer = Layer::new()
        .with_writer(non_blocking_file)
        .with_ansi(false)
        .with_target(true)
        .with_level(true)
        .with_thread_ids(false)
        .with_thread_names(false);

    let console_log_layer = Layer::new()
        .with_writer(non_blocking_console)
        .with_ansi(true)
        .with_target(true)
        .with_level(true)
        .with_thread_ids(false)
        .with_thread_names(false);

    let mut layers = Vec::new();
    if config.logging.log_to_file {
        layers.push(file_log_layer);
    }
    if config.logging.log_to_console {
        layers.push(console_log_layer);
    }
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            config.logging.level.clone(),
        ))
        .with(layers)
        .init();
    Ok((_console_guard, _file_guard))
}
