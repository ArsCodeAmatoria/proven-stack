use std::net::SocketAddr;

use anyhow::Context;
use proven_platform::{build_app, init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let host = std::env::var("PROVEN_API_HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = std::env::var("PROVEN_API_PORT")
        .unwrap_or_else(|_| "8080".into())
        .parse()
        .context("invalid PROVEN_API_PORT")?;

    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .context("invalid bind address")?;
    let app = build_app();

    tracing::info!(%addr, "proven-api listening");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("failed to bind")?;
    axum::serve(listener, app).await.context("server error")?;
    Ok(())
}
