use std::net::SocketAddr;

use anyhow::Context;
use proven_config::load;
use proven_platform::{build_app, init_tracing, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = load().context("configuration load failed")?;
    let observability = init_tracing(&config).context("observability init failed")?;

    tracing::info!(
        environment = %config.environment,
        service = %config.observability.service_name,
        version = %config.observability.service_version,
        bind = %config.server.bind_addr(),
        infra_optional = config.infra.optional,
        metrics = config.observability.metrics_enabled,
        otel = config.observability.otel_enabled,
        "configuration loaded"
    );
    tracing::debug!(?config, "typed configuration");

    let addr: SocketAddr = config
        .server
        .bind_addr()
        .parse()
        .context("invalid bind address from configuration")?;

    let state = AppState::connect(config, observability.metrics.clone())
        .await
        .context("application state / infrastructure bootstrap failed")?;
    let app = build_app(state);

    tracing::info!(%addr, "proven-api listening");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("failed to bind")?;
    axum::serve(listener, app).await.context("server error")?;

    drop(observability);
    Ok(())
}
