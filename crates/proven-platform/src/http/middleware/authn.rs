//! Authentication conventions middleware (ADR-0002 / ADR-0013).
//!
//! Documents and lightly enforces auth transport for versioned API routes:
//! - Preferred: `Authorization: Bearer <token>` (Better Auth / JWT — adapter pending)
//! - Interim DX: `X-Proven-Tenant-Id` + `X-Proven-User-Id`
//!
//! Public paths (health, docs, openapi) are skipped. Missing credentials on
//! protected `/api/v1/*` routes yield `401` with the nested error envelope.
//! Authorization (permissions) remains fail-closed via [`super::authz`].

use axum::extract::Request;
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use proven_shared::AppError;

use crate::http::ApiError;

fn is_public(path: &str) -> bool {
    matches!(
        path,
        "/health"
            | "/healthz"
            | "/readyz"
            | "/metrics"
            | "/docs"
            | "/redoc"
            | "/api-docs/openapi.json"
            | "/api/v1/openapi.json"
            | "/api/v1/health"
            | "/api/v1/health/db"
            | "/api/v1/health/temporal"
            | "/api/v1/db/version"
    ) || path.starts_with("/docs/")
}

fn has_credentials(request: &Request) -> bool {
    if request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.to_ascii_lowercase().starts_with("bearer "))
    {
        return true;
    }
    let tenant = request
        .headers()
        .get("x-proven-tenant-id")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty());
    let user = request
        .headers()
        .get("x-proven-user-id")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty());
    tenant.is_some() && user.is_some()
}

/// When `enforce` is true, reject unauthenticated calls to `/api/v1/*` (except public).
pub async fn authentication_layer(request: Request, next: Next) -> Response {
    let path = request.uri().path().to_string();
    let enforce = request
        .extensions()
        .get::<AuthnPolicy>()
        .map(|p| p.enforce_credentials)
        .unwrap_or(false);

    if enforce && path.starts_with("/api/v1/") && !is_public(&path) && !has_credentials(&request)
    {
        return ApiError::from(AppError::Unauthorized).into_response();
    }

    next.run(request).await
}

/// Toggle for credential enforcement (off in unit tests by default).
#[derive(Clone, Debug, Default)]
pub struct AuthnPolicy {
    pub enforce_credentials: bool,
}
