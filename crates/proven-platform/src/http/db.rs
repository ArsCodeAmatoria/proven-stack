//! Database health and version endpoints (no business schema).

use axum::extract::State;
use axum::Json;
use proven_db::{check_health, database_version, DatabaseVersion, DbHealth};
use proven_shared::AppError;
use serde::Serialize;
use utoipa::ToSchema;

use crate::http::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub struct DbHealthEnvelope {
    pub data: DbHealthBody,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DbHealthBody {
    pub ok: bool,
    pub latency_ms: u128,
    pub detail: String,
    pub pool_attached: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DbVersionEnvelope {
    pub data: DatabaseVersionBody,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DatabaseVersionBody {
    pub postgres_version: String,
    pub migration_version: Option<i64>,
    pub migration_description: Option<String>,
    pub migration_installed_on: Option<String>,
    pub migration_success: Option<bool>,
    pub applied_migrations: i64,
}

impl From<DbHealth> for DbHealthBody {
    fn from(value: DbHealth) -> Self {
        Self {
            ok: value.ok,
            latency_ms: value.latency_ms,
            detail: value.detail,
            pool_attached: true,
        }
    }
}

impl From<DatabaseVersion> for DatabaseVersionBody {
    fn from(value: DatabaseVersion) -> Self {
        Self {
            postgres_version: value.postgres_version,
            migration_version: value.migration_version,
            migration_description: value.migration_description,
            migration_installed_on: value
                .migration_installed_on
                .map(|ts| ts.to_rfc3339()),
            migration_success: value.migration_success,
            applied_migrations: value.applied_migrations,
        }
    }
}

/// `GET /api/v1/health/db` — live Postgres pool probe.
#[utoipa::path(
    get,
    path = "/api/v1/health/db",
    tag = "database",
    responses(
        (status = 200, description = "Database health", body = DbHealthEnvelope),
        (status = 503, description = "Database unavailable")
    )
)]
pub async fn db_health(State(state): State<AppState>) -> Result<Json<DbHealthEnvelope>, ApiError> {
    let Some(pool) = state.db() else {
        return Err(AppError::Unavailable("postgres pool not configured".into()).into());
    };

    let health = check_health(pool).await;
    let body = DbHealthEnvelope {
        data: DbHealthBody::from(health.clone()),
    };
    if health.ok {
        Ok(Json(body))
    } else {
        Err(AppError::Unavailable(health.detail).into())
    }
}

/// `GET /api/v1/db/version` — Postgres + migration metadata versions.
#[utoipa::path(
    get,
    path = "/api/v1/db/version",
    tag = "database",
    responses(
        (status = 200, description = "Database and migration versions", body = DbVersionEnvelope),
        (status = 503, description = "Database unavailable")
    )
)]
pub async fn db_version(State(state): State<AppState>) -> Result<Json<DbVersionEnvelope>, ApiError> {
    let Some(pool) = state.db() else {
        return Err(AppError::Unavailable("postgres pool not configured".into()).into());
    };

    let version = database_version(pool)
        .await
        .map_err(|e| AppError::Unavailable(e.to_string()))?;

    Ok(Json(DbVersionEnvelope {
        data: DatabaseVersionBody::from(version),
    }))
}
