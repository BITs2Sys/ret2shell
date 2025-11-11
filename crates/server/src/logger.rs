//! This module setup the logger of `tracing`.
//!
//! In the initialization stage, the log writer will be guarded by their file
//! descriptor, the lifetime of the file descriptor is the same as the log
//! writer. You should keep the file descriptor until application exit.

use std::{
  io::{Result as IoResult, Write},
  path::Path,
  time::Duration,
};

use axum::{body::Body, http::Request};
use hyper_util::{
  client::legacy::{Client, connect::HttpConnector},
  rt::TokioExecutor,
};
use r2s_config::logging;
use thiserror::Error;
use tokio::{
  sync::mpsc,
  time::{MissedTickBehavior, interval},
};
use tracing::error;
use tracing_appender::{non_blocking, non_blocking::WorkerGuard, rolling};
use tracing_subscriber::{
  EnvFilter,
  fmt::{Layer, MakeWriter},
  prelude::*,
};

#[derive(Error, Debug)]
pub enum LoggerError {
  #[error("io error: {0}")]
  Io(#[from] std::io::Error),
  #[error("file logger init error")]
  FileLoggerInitError(#[from] rolling::InitError),
}

// A MakeWriter that provides a per-event writer, which pushes complete JSON
// lines to a background worker.
#[derive(Clone)]
struct VlMakeWriter {
  tx: mpsc::Sender<Vec<u8>>,
}

impl VlMakeWriter {
  fn new(tx: mpsc::Sender<Vec<u8>>) -> Self {
    Self { tx }
  }
}

struct VlLineWriter {
  tx: mpsc::Sender<Vec<u8>>,
  buf: Vec<u8>,
}

impl Write for VlLineWriter {
  fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
    // tracing-subscriber JSON formatter writes one JSON object per
    // event/span-event, typically ending with '\n'. We split on newlines and
    // send complete lines.
    self.buf.extend_from_slice(buf);

    while let Some(pos) = self.buf.iter().position(|&b| b == b'\n') {
      let mut line = self.buf.drain(..=pos).collect::<Vec<u8>>();
      if !line.ends_with(b"\n") {
        line.push(b'\n');
      }
      // Backpressure: block if full to avoid dropping logs
      if let Err(e) = self.tx.blocking_send(line) {
        error!(error=?e, "failed to enqueue log line for victoria logs server");
      }
    }

    Ok(buf.len())
  }

  fn flush(&mut self) -> IoResult<()> {
    if !self.buf.is_empty() {
      let mut line = self.buf.split_off(0);
      if !line.ends_with(b"\n") {
        line.push(b'\n');
      }
      if let Err(e) = self.tx.blocking_send(line) {
        error!(
          error=?e,
          "failed to enqueue partial log line for victoria logs server",
        );
      }
    }
    Ok(())
  }
}

impl<'a> MakeWriter<'a> for VlMakeWriter {
  type Writer = VlLineWriter;

  fn make_writer(&'a self) -> Self::Writer {
    VlLineWriter {
      tx: self.tx.clone(),
      buf: Vec::with_capacity(1024),
    }
  }
}

// Background task that batches and POSTs ndjson to VictoriaLogs via hyper_util
// legacy client
async fn run_vl_worker(
  mut rx: mpsc::Receiver<Vec<u8>>, url: String, client: Client<HttpConnector, Body>,
) {
  let mut ticker = interval(Duration::from_secs(5));
  ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

  let mut body: Vec<u8> = Vec::with_capacity(4096);
  let mut lines = 0usize;

  let flush = |mut body: Vec<u8>| async {
    if body.is_empty() {
      return;
    }
    // Ensure final body ends with a newline (VictoriaLogs accepts NDJSON lines)
    if !body.ends_with(b"\n") {
      body.extend_from_slice(b"\n");
    }

    let mut builder = Request::builder().method("POST").uri(format!(
      "{}/insert/jsonline?_time_field=timestamp&_msg_field=fields.message",
      url
    ));

    // Content-Type is text/plain for NDJSON; application/stream+json also works
    builder = builder.header("Content-Type", "application/stream+json");

    let body = Body::from(body);

    let req = match builder.body(body) {
      Ok(r) => r,
      Err(err) => {
        error!(error=?err, "failed to build request for victoria logs server");
        return;
      }
    };

    // Send and consume the response body to allow connection reuse
    match client.request(req).await {
      Ok(resp) => {
        if !resp.status().is_success() {
          error!(
            ?resp,
            "remote victoria logs server returned non-success status"
          );
        }
      }
      Err(err) => {
        error!(error=?err, "failed to POST batch");
      }
    }
  };

  loop {
    tokio::select! {
        maybe_line = rx.recv() => {
            match maybe_line {
                Some(mut line) => {
                    if !line.ends_with(b"\n") {
                        line.push(b'\n');
                    }
                    body.extend_from_slice(&line);
                    lines += 1;

                    if body.len() >= 3072 || lines >= 100 {
                        flush(body).await;
                        body = Vec::with_capacity(4096);
                        lines = 0;
                    }
                }
                None => {
                    flush(body).await;
                    break;
                }
            }
        }
        _ = ticker.tick() => {
            flush(body).await;
            body = Vec::with_capacity(4096);
            lines = 0;
        }
    }
  }
}
/// Initialize the logger.
pub async fn initialize(config: &Option<logging::Config>) -> Result<Vec<WorkerGuard>, LoggerError> {
  let config = config.clone().unwrap_or(logging::Config {
    level: "info".to_string(),
    directory: "./log".to_string(),
    files_kept: None,
    compress: Some(false),
    victoria: None,
  });
  let logging::Config {
    level,
    directory,
    files_kept,
    compress: _,
    victoria,
  } = config;
  tokio::fs::create_dir_all(&directory).await?;
  let mut file_appender = rolling::RollingFileAppender::builder()
    .rotation(rolling::Rotation::DAILY)
    .filename_prefix("ret2shell")
    .filename_suffix("log");
  if let Some(files_kept) = files_kept {
    file_appender = file_appender.max_log_files(files_kept);
  }
  let file_appender = file_appender.build(Path::new(&directory).canonicalize()?)?;

  let (non_blocking_file, file_guard) = non_blocking(file_appender);
  let (non_blocking_console, console_guard) = non_blocking(std::io::stdout());
  let file_log_layer = Layer::new()
    .with_writer(non_blocking_file)
    .with_ansi(false)
    .with_target(true)
    .with_level(true)
    .with_thread_ids(false)
    .with_thread_names(false)
    .json();

  let console_log_layer = Layer::new()
    .with_writer(non_blocking_console)
    .with_ansi(true)
    .with_target(true)
    .with_level(true)
    .with_thread_ids(false)
    .with_thread_names(false);

  if let Some(victoria_url) = victoria {
    let client: Client<_, Body> = Client::builder(TokioExecutor::new()).build(HttpConnector::new());

    let (tx, rx) = mpsc::channel::<Vec<u8>>(64_000);
    // Spawn background worker
    let worker_client = client.clone();
    tokio::spawn(async move { run_vl_worker(rx, victoria_url, worker_client).await });

    // Hook tracing to VictoriaLogs writer (as a Layer so we can flatten events and
    // add span events)
    let make_writer = VlMakeWriter::new(tx);
    let (non_blocking_victoria, victoria_guard) = non_blocking(make_writer.make_writer());
    let victoria_log_layer = Layer::new()
      .with_writer(non_blocking_victoria)
      .with_ansi(false)
      .with_target(true)
      .with_level(true)
      .with_thread_ids(false)
      .with_thread_names(false)
      .json();
    tracing_subscriber::registry()
      .with(EnvFilter::new(level))
      .with(file_log_layer)
      .with(console_log_layer)
      .with(victoria_log_layer)
      .init();
    Ok(vec![console_guard, file_guard, victoria_guard])
  } else {
    tracing_subscriber::registry()
      .with(EnvFilter::new(level))
      .with(file_log_layer)
      .with(console_log_layer)
      .init();
    Ok(vec![console_guard, file_guard])
  }
}
