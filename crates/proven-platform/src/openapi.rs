//! OpenAPI document for the platform host (foundation paths only).

use utoipa::OpenApi;

use crate::http::db::{DatabaseVersionBody, DbHealthBody, DbHealthEnvelope, DbVersionEnvelope};
use crate::http::health::{ApiHealthEnvelope, HealthResponse, ReadyResponse};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Proven API",
        version = "0.1.0",
        description = "Construction Compliance Operating System — foundation host (no domain modules yet)."
    ),
    paths(
        crate::http::health::health,
        crate::http::health::readyz,
        crate::http::health::api_health,
        crate::http::db::db_health,
        crate::http::db::db_version,
    ),
    components(schemas(
        HealthResponse,
        ReadyResponse,
        ApiHealthEnvelope,
        crate::http::health::ApiHealthData,
        DbHealthEnvelope,
        DbHealthBody,
        DbVersionEnvelope,
        DatabaseVersionBody
    )),
    tags(
        (name = "platform", description = "Host health and readiness"),
        (name = "database", description = "PostgreSQL health and migration version")
    )
)]
pub struct ApiDoc;
