//! Liveness / readiness endpoints (no business logic).

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use proven_shared::HealthStatus;
use serde::Serialize;
use utoipa::ToSchema;

use crate::http::error::ApiError;
use crate::state::AppState;
use proven_shared::AppError;

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReadyResponse {
    pub status: String,
    pub service: String,
    pub postgres: bool,
    pub redis: bool,
    pub nats: bool,
    pub temporal: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiHealthData {
    pub status: String,
    pub version: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiHealthEnvelope {
    pub data: ApiHealthData,
}

/// `GET /health` — always HTTP 200 when the process is up.
#[utoipa::path(
    get,
    path = "/health",
    tag = "platform",
    responses(
        (status = 200, description = "Process is alive", body = HealthResponse)
    )
)]
pub async fn health() -> (StatusCode, Json<HealthResponse>) {
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "ok".into(),
            service: "proven-api".into(),
        }),
    )
}

/// Kubernetes-style alias.
pub async fn healthz() -> Json<HealthStatus> {
    Json(HealthStatus {
        status: "ok".into(),
        service: "proven-api".into(),
    })
}

/// Readiness — requires infra when configured as mandatory.
#[utoipa::path(
    get,
    path = "/readyz",
    tag = "platform",
    responses(
        (status = 200, description = "Ready to serve", body = ReadyResponse),
        (status = 503, description = "Dependencies unavailable")
    )
)]
pub async fn readyz(State(state): State<AppState>) -> Result<Json<ReadyResponse>, ApiError> {
    let postgres = state.postgres_healthy().await;
    let body = ReadyResponse {
        status: if postgres
            && state.redis().is_some()
            && state.nats().is_some()
            && state.temporal().is_some()
        {
            "ready".into()
        } else {
            "degraded".into()
        },
        service: "proven-api".into(),
        postgres,
        redis: state.redis().is_some(),
        nats: state.nats().is_some(),
        temporal: state.temporal().is_some(),
    };

    let fully_ready = postgres
        && state.redis().is_some()
        && state.nats().is_some()
        && state.temporal().is_some();

    if state.config().infra.optional || fully_ready {
        Ok(Json(body))
    } else {
        Err(AppError::Unavailable("infrastructure not ready".into()).into())
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "platform",
    responses(
        (status = 200, description = "API health envelope", body = ApiHealthEnvelope)
    )
)]
pub async fn api_health() -> Json<ApiHealthEnvelope> {
    Json(ApiHealthEnvelope {
        data: ApiHealthData {
            status: "ok".into(),
            version: env!("CARGO_PKG_VERSION").into(),
        },
    })
}
