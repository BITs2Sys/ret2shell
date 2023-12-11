//! Provides TCP <-> WS proxy service for players.
use axum::extract::ws::WebSocket;
use tokio::net::TcpStream;
use tracing::error;
use wsrx::proxy_axum_ws;

pub async fn connect(tcp_addr: impl AsRef<str>, ws_conn: WebSocket) -> anyhow::Result<()> {
    let tcp = TcpStream::connect(tcp_addr.as_ref()).await?;
    tokio::spawn(async move {
        match proxy_axum_ws(ws_conn, tcp).await {
            Ok(_) => {}
            Err(e) => {
                error!("proxying websocket error: {}", e);
            }
        };
    });
    Ok(())
}
