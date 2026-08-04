//! Temporal health HTTP endpoint.

use axum::extract::State;
use axum::Json;
use proven_shared::AppError;
use proven_temporal::TemporalHealthStatus;
use serde::Serialize;
use utoipa::ToSchema;

use crate::http::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub struct TemporalHealthEnvelope {
    pub data: TemporalHealthBody,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TemporalHealthBody {
    pub status: String,
    pub address: String,
    pub namespace: String,
    pub reachable: bool,
    pub workflow_definitions: usize,
    pub activity_definitions: usize,
    pub worker_running: bool,
    pub task_queue: String,
    pub detail: String,
}

/// `GET /api/v1/health/temporal` — Temporal reachability + empty registry status.
#[utoipa::path(
    get,
    path = "/api/v1/health/temporal",
    tag = "platform",
    responses(
        (status = 200, description = "Temporal health", body = TemporalHealthEnvelope),
        (status = 503, description = "Temporal unavailable")
    )
)]
pub async fn temporal_health(
    State(state): State<AppState>,
) -> Result<Json<TemporalHealthEnvelope>, ApiError> {
    let Some(handle) = state.temporal() else {
        return Err(AppError::Unavailable("temporal client not configured".into()).into());
    };

    let health = handle.health().await;
    let worker = handle.worker().status();
    let body = TemporalHealthEnvelope {
        data: TemporalHealthBody {
            status: match health.status {
                TemporalHealthStatus::Healthy => "healthy".into(),
                TemporalHealthStatus::Degraded => "degraded".into(),
                TemporalHealthStatus::Unavailable => "unavailable".into(),
            },
            address: health.address,
            namespace: health.namespace,
            reachable: health.reachable,
            workflow_definitions: health.workflow_definitions,
            activity_definitions: health.activity_definitions,
            worker_running: worker.running,
            task_queue: worker.task_queue,
            detail: health.detail.clone(),
        },
    };

    if health.reachable {
        Ok(Json(body))
    } else {
        Err(AppError::Unavailable(health.detail).into())
    }
}
