//! Prometheus scrape endpoint.

use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use proven_observability::render_metrics;

use crate::state::AppState;

/// `GET /metrics` — Prometheus text exposition (foundation infra only).
pub async fn metrics(State(state): State<AppState>) -> impl IntoResponse {
    match state.metrics() {
        Some(handle) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/plain; version=0.0.4")],
            render_metrics(handle),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            "metrics disabled (PROVEN_METRICS_ENABLED=false)",
        )
            .into_response(),
    }
}
