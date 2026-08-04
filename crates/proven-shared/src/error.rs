//! Platform-level application errors (no domain semantics).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Field-level validation / domain detail (REST_API.md §12.2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct FieldError {
    pub field: String,
    pub code: String,
    pub message: String,
}

impl FieldError {
    pub fn new(
        field: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            code: code.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("validation failed: {message}")]
    Validation {
        message: String,
        details: Vec<FieldError>,
    },
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("precondition failed: {0}")]
    PreconditionFailed(String),
    #[error("rate limited")]
    RateLimited { retry_after_secs: u64, limit: u32 },
    #[error("service unavailable: {0}")]
    Unavailable(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    pub fn validation(message: impl Into<String>, details: Vec<FieldError>) -> Self {
        Self::Validation {
            message: message.into(),
            details,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            Self::NotFound => "not_found",
            Self::Unauthorized => "unauthorized",
            Self::Forbidden => "forbidden",
            Self::BadRequest(_) => "bad_request",
            Self::Validation { .. } => "validation_failed",
            Self::Conflict(_) => "conflict",
            Self::PreconditionFailed(_) => "precondition_failed",
            Self::RateLimited { .. } => "rate_limited",
            Self::Unavailable(_) => "unavailable",
            Self::Internal(_) => "internal",
        }
    }

    pub fn status_code(&self) -> u16 {
        match self {
            Self::NotFound => 404,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::BadRequest(_) => 400,
            Self::Validation { .. } => 422,
            Self::Conflict(_) => 409,
            Self::PreconditionFailed(_) => 412,
            Self::RateLimited { .. } => 429,
            Self::Unavailable(_) => 503,
            Self::Internal(_) => 500,
        }
    }

    pub fn field_errors(&self) -> Vec<FieldError> {
        match self {
            Self::Validation { details, .. } => details.clone(),
            _ => Vec::new(),
        }
    }
}
