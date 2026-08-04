//! OpenAPI document for the platform host + REST convention schemas (ADR-0013).

use axum::Json;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::http::db::{DatabaseVersionBody, DbHealthBody, DbHealthEnvelope, DbVersionEnvelope};
use crate::http::health::{ApiHealthData, ApiHealthEnvelope, HealthResponse, ReadyResponse};
use crate::http::temporal::{TemporalHealthBody, TemporalHealthEnvelope};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Proven API",
        version = "1.0.0",
        description = "Construction Compliance Operating System — `/api/v1` REST surface. Conventions: nested `{ error }` / `{ data }` envelopes, cursor pagination, Bearer + interim header AuthN, Core AuthZ, rate limits (ADR-0013)."
    ),
    paths(
        crate::http::health::health,
        crate::http::health::readyz,
        crate::http::health::api_health,
        crate::http::db::db_health,
        crate::http::db::db_version,
        crate::http::temporal::temporal_health,
        openapi_json,
    ),
    components(schemas(
        HealthResponse,
        ReadyResponse,
        ApiHealthEnvelope,
        ApiHealthData,
        DbHealthEnvelope,
        DbHealthBody,
        DbVersionEnvelope,
        DatabaseVersionBody,
        TemporalHealthEnvelope,
        TemporalHealthBody,
        proven_shared::ErrorResponse,
        proven_shared::ErrorBody,
        proven_shared::FieldError,
        proven_shared::PaginationMeta,
    )),
    tags(
        (name = "platform", description = "Host health, readiness, OpenAPI"),
        (name = "database", description = "PostgreSQL health and migration version")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearerAuth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some(
                        "Better Auth / JWT access token (Authorization: Bearer …)",
                    ))
                    .build(),
            ),
        );
        components.add_security_scheme(
            "apiKeyAuth",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-Api-Key"))),
        );
        components.add_security_scheme(
            "interimTenant",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-Proven-Tenant-Id"))),
        );
        components.add_security_scheme(
            "interimUser",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-Proven-User-Id"))),
        );
    }
}

/// `GET /api/v1/openapi.json` — versioned OpenAPI document (REST_API.md §14).
#[utoipa::path(
    get,
    path = "/api/v1/openapi.json",
    tag = "platform",
    responses(
        (status = 200, description = "OpenAPI 3 document")
    )
)]
pub async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
