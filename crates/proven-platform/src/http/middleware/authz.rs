//! Platform-wide AuthZ middleware helper (ADR-0007 §9: "Platform AuthZ middleware requires a
//! permission + scope for protected routes; fail closed").
//!
//! `proven-core`'s `AuthzApi` remains the **only** decision authority (ADR-0003) — this module
//! is a thin transport-layer adapter, not a second AuthZ implementation. It:
//!
//! 1. Extracts the acting tenant/user from `X-Proven-Tenant-Id` / `X-Proven-User-Id` (the same
//!    interim scheme `proven-core::api::extractors::CorePrincipal` uses — ADR-0002; production
//!    replaces this with a validated Better Auth session).
//! 2. Calls `AppState::core().services.authorize(...)` for a given permission + scope.
//! 3. Maps `Deny` to `403 Forbidden` and any transport/internal error to the platform
//!    `ApiError` (RFC-7807-ish problem body).
//!
//! ## Usage
//!
//! Call [`require_permission`] at the top of a handler (or wrap it in a small
//! `axum::middleware::from_fn_with_state` adapter if a route needs it unconditionally) before
//! performing the protected action:
//!
//! ```ignore
//! use proven_platform::http::middleware::{require_permission, AuthzPrincipal};
//!
//! pub async fn admin_only_handler(
//!     State(state): State<AppState>,
//!     principal: AuthzPrincipal,
//! ) -> Result<StatusCode, ApiError> {
//!     require_permission(
//!         &state,
//!         &principal,
//!         "core.tenant.manage",
//!         AccessScope::tenant(),
//!     )
//!     .await?;
//!     // ... perform the protected action ...
//!     Ok(StatusCode::NO_CONTENT)
//! }
//! ```

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use proven_core::application::services::AuthorizeRequest;
use proven_core::domain::AccessScope;
use proven_core::AuthzApi;
use proven_shared::{AppError, PrincipalId, ProblemDetails, TenantId, UserId};
use uuid::Uuid;

use crate::state::AppState;

pub const TENANT_HEADER: &str = "x-proven-tenant-id";
pub const USER_HEADER: &str = "x-proven-user-id";

/// The acting tenant + user resolved from interim dev headers — platform-level analogue of
/// `proven_core::api::extractors::CorePrincipal`, usable by any module's Axum handlers.
#[derive(Debug, Clone, Copy)]
pub struct AuthzPrincipal {
    pub tenant_id: TenantId,
    pub user_id: UserId,
}

impl AuthzPrincipal {
    pub fn principal_id(&self) -> PrincipalId {
        PrincipalId::from_uuid(self.user_id.as_uuid())
    }
}

/// Rejection wrapper so extractor failures still render as `ProblemDetails`.
pub struct AuthzPrincipalRejection(AppError);

impl IntoResponse for AuthzPrincipalRejection {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.0.status_code()).unwrap_or(StatusCode::BAD_REQUEST);
        (status, Json(ProblemDetails::from(&self.0))).into_response()
    }
}

impl<S> FromRequestParts<S> for AuthzPrincipal
where
    S: Send + Sync,
{
    type Rejection = AuthzPrincipalRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let tenant_id = header_uuid(parts, TENANT_HEADER)?;
        let user_id = header_uuid(parts, USER_HEADER)?;
        Ok(AuthzPrincipal {
            tenant_id: TenantId::from_uuid(tenant_id),
            user_id: UserId::from_uuid(user_id),
        })
    }
}

fn header_uuid(parts: &Parts, name: &'static str) -> Result<Uuid, AuthzPrincipalRejection> {
    let value = parts
        .headers
        .get(name)
        .ok_or(AuthzPrincipalRejection(AppError::Unauthorized))?
        .to_str()
        .map_err(|_| {
            AuthzPrincipalRejection(AppError::BadRequest(format!("invalid {name} header")))
        })?;
    Uuid::parse_str(value)
        .map_err(|_| AuthzPrincipalRejection(AppError::BadRequest(format!("invalid {name} header"))))
}

/// Call Core's `AuthzApi::authorize` for `permission` at `scope`, mapping `Deny` to
/// `403 Forbidden` and any repository/internal error to its platform `ApiError` — fail closed
/// (ADR-0003, ADR-0007 §9). No ABAC context is threaded through here; callers needing ABAC
/// signals (resource state, assurance level) should call `AuthzApi::authorize` directly with a
/// populated `AuthorizeRequest`.
pub async fn require_permission(
    state: &AppState,
    principal: &AuthzPrincipal,
    permission: &str,
    scope: AccessScope,
) -> Result<(), crate::http::ApiError> {
    let decision = state
        .core()
        .services
        .authorize(AuthorizeRequest::new(
            principal.tenant_id,
            principal.principal_id(),
            permission.into(),
            scope,
        ))
        .await
        .map_err(|err| crate::http::ApiError::from(AppError::Internal(err.to_string())))?;

    if decision.is_allowed() {
        Ok(())
    } else {
        Err(crate::http::ApiError::from(AppError::Forbidden))
    }
}
