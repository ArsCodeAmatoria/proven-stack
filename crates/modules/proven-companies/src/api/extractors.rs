//! Interim principal extraction from `X-Proven-Tenant-Id` / `X-Proven-User-Id` headers — copied
//! from `proven_core::api::extractors::CorePrincipal` so this module doesn't depend on Core's
//! HTTP internals.
//!
//! ADR-0002: acceptable for non-production smoke tests only. Production must resolve the
//! principal from a validated Better Auth session (`sub`, `tid`, `sid` claims) instead.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use proven_shared::{AppError, PrincipalId, ProblemDetails, TenantId, UserId};
use uuid::Uuid;

use crate::application::services::ActingContext;

pub const TENANT_HEADER: &str = "x-proven-tenant-id";
pub const USER_HEADER: &str = "x-proven-user-id";

/// The acting tenant + user resolved from interim dev headers.
#[derive(Debug, Clone, Copy)]
pub struct CompaniesPrincipal {
    pub tenant_id: TenantId,
    pub user_id: UserId,
}

impl CompaniesPrincipal {
    pub fn principal_id(&self) -> PrincipalId {
        PrincipalId::from_uuid(self.user_id.as_uuid())
    }

    pub fn acting_context(&self) -> ActingContext {
        ActingContext::new(self.tenant_id, self.principal_id())
    }
}

/// Rejection wrapper so extractor failures still render as `ProblemDetails`.
pub struct ExtractorRejection(AppError);

impl IntoResponse for ExtractorRejection {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.0.status_code()).unwrap_or(StatusCode::BAD_REQUEST);
        let body = ProblemDetails::from(&self.0);
        (status, Json(body)).into_response()
    }
}

impl<S> FromRequestParts<S> for CompaniesPrincipal
where
    S: Send + Sync,
{
    type Rejection = ExtractorRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let tenant_id = header_uuid(parts, TENANT_HEADER)?;
        let user_id = header_uuid(parts, USER_HEADER)?;
        Ok(CompaniesPrincipal {
            tenant_id: TenantId::from_uuid(tenant_id),
            user_id: UserId::from_uuid(user_id),
        })
    }
}

fn header_uuid(parts: &Parts, name: &'static str) -> Result<Uuid, ExtractorRejection> {
    let value = parts
        .headers
        .get(name)
        .ok_or(ExtractorRejection(AppError::Unauthorized))?
        .to_str()
        .map_err(|_| ExtractorRejection(AppError::BadRequest(format!("invalid {name} header"))))?;
    Uuid::parse_str(value)
        .map_err(|_| ExtractorRejection(AppError::BadRequest(format!("invalid {name} header"))))
}
