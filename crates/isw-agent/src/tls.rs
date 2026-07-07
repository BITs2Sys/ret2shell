//! Optional mTLS listener for the agent. Builds a rustls `ServerConfig` that
//! requires a client certificate signed by the configured CA, then serves the
//! axum app over tokio-rustls via hyper-util (no `axum-server` dependency).

use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use axum::Router;
use base64::{Engine, engine::general_purpose::STANDARD};
use hyper_util::{
  rt::{TokioExecutor, TokioIo},
  server::conn::auto::Builder,
  service::TowerToHyperService,
};
use rustls::{
  RootCertStore, ServerConfig,
  pki_types::{CertificateDer, PrivateKeyDer},
  server::WebPkiClientVerifier,
};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

use crate::registry::TlsConfig;

/// Extract the DER bodies of every `-----BEGIN <tag>-----` block in a PEM string.
fn pem_blocks(pem: &str, tag: &str) -> Vec<Vec<u8>> {
  let begin = format!("-----BEGIN {tag}-----");
  let end = format!("-----END {tag}-----");
  let mut out = Vec::new();
  let mut rest = pem;
  while let Some(s) = rest.find(&begin) {
    let after = &rest[s + begin.len()..];
    let Some(e) = after.find(&end) else { break };
    let body: String = after[..e].chars().filter(|c| !c.is_whitespace()).collect();
    if let Ok(der) = STANDARD.decode(body.as_bytes()) {
      out.push(der);
    }
    rest = &after[e + end.len()..];
  }
  out
}

fn load_certs(pem: &str) -> Result<Vec<CertificateDer<'static>>> {
  let certs: Vec<_> = pem_blocks(pem, "CERTIFICATE")
    .into_iter()
    .map(CertificateDer::from)
    .collect();
  if certs.is_empty() {
    return Err(anyhow!("no CERTIFICATE blocks found"));
  }
  Ok(certs)
}

fn load_key(pem: &str) -> Result<PrivateKeyDer<'static>> {
  // support PKCS#8 ("PRIVATE KEY"), then RSA / EC fallbacks.
  for tag in ["PRIVATE KEY", "RSA PRIVATE KEY", "EC PRIVATE KEY"] {
    if let Some(der) = pem_blocks(pem, tag).into_iter().next() {
      return PrivateKeyDer::try_from(der).map_err(|e| anyhow!("invalid private key: {e}"));
    }
  }
  Err(anyhow!("no PRIVATE KEY block found"))
}

fn build_server_config(tls: &TlsConfig) -> Result<ServerConfig> {
  let cert_pem = std::fs::read_to_string(&tls.cert)
    .with_context(|| format!("read tls cert {}", tls.cert))?;
  let key_pem =
    std::fs::read_to_string(&tls.key).with_context(|| format!("read tls key {}", tls.key))?;
  let ca_pem =
    std::fs::read_to_string(&tls.ca).with_context(|| format!("read tls ca {}", tls.ca))?;

  let certs = load_certs(&cert_pem)?;
  let key = load_key(&key_pem)?;

  let mut roots = RootCertStore::empty();
  for ca in load_certs(&ca_pem)? {
    roots
      .add(ca)
      .map_err(|e| anyhow!("adding CA to root store: {e}"))?;
  }
  let verifier = WebPkiClientVerifier::builder(Arc::new(roots))
    .build()
    .map_err(|e| anyhow!("building client verifier: {e}"))?;

  ServerConfig::builder()
    .with_client_cert_verifier(verifier)
    .with_single_cert(certs, key)
    .map_err(|e| anyhow!("building server config: {e}"))
}

/// Serve the axum app over mTLS. Each accepted TCP connection is TLS-handshaked
/// (rejecting clients without a CA-signed cert) then handed to hyper-util.
pub async fn serve(listen: &str, app: Router, tls: &TlsConfig) -> Result<()> {
  // install the ring crypto provider for this process (idempotent).
  let _ = rustls::crypto::ring::default_provider().install_default();
  let config = build_server_config(tls)?;
  let acceptor = TlsAcceptor::from(Arc::new(config));
  let listener = TcpListener::bind(listen)
    .await
    .with_context(|| format!("failed to bind {listen}"))?;
  tracing::info!(%listen, "isw-agent listening (mTLS)");
  loop {
    let (tcp, _peer) = match listener.accept().await {
      Ok(pair) => pair,
      Err(err) => {
        tracing::warn!(error = %err, "accept failed");
        continue;
      }
    };
    let acceptor = acceptor.clone();
    let service = TowerToHyperService::new(app.clone());
    tokio::spawn(async move {
      let tls = match acceptor.accept(tcp).await {
        Ok(tls) => tls,
        Err(err) => {
          tracing::debug!(error = %err, "tls handshake failed");
          return;
        }
      };
      let io = TokioIo::new(tls);
      if let Err(err) = Builder::new(TokioExecutor::new())
        .serve_connection_with_upgrades(io, service)
        .await
      {
        tracing::debug!(error = %err, "connection error");
      }
    });
  }
}
