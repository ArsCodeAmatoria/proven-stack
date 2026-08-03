//! Proven platform host — router, health, middleware wiring.
//! Business modules are registered here in later milestones.

use axum::{routing::get, Json, Router};
use proven_shared::HealthStatus;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub fn build_app() -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/api/v1/health", get(api_health))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}

async fn healthz() -> Json<HealthStatus> {
    Json(HealthStatus {
        status: "ok",
        service: "proven-api",
    })
}

async fn readyz() -> Json<HealthStatus> {
    // Foundation: no external deps required yet.
    Json(HealthStatus {
        status: "ready",
        service: "proven-api",
    })
}

async fn api_health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "data": {
            "status": "ok",
            "version": env!("CARGO_PKG_VERSION")
        }
    }))
}

pub fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,tower_http=info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();
}
