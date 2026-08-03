//! Proven platform host — Axum router, AppState DI, infra adapters.
//! Business domain modules are registered in later milestones.

pub mod http;
pub mod infra;
pub mod openapi;
pub mod state;
pub mod tracing_init;

pub use http::build_router;
pub use state::AppState;
pub use tracing_init::init_tracing;

use proven_config::Config;
use proven_observability::ObservabilityHandle;

/// Build the Axum application with shared state (dependency injection root).
pub fn build_app(state: AppState) -> axum::Router {
    build_router(state)
}

/// Convenience: construct observability + infra + router from config.
pub async fn build_app_from_config(
    config: Config,
) -> anyhow::Result<(axum::Router, ObservabilityHandle)> {
    let obs = init_tracing(&config)?;
    let state = AppState::connect(config, obs.metrics.clone()).await?;
    Ok((build_app(state), obs))
}
