//! Nested error envelope (`{ error: { code, message, details, correlation_id } }`).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::{AppError, FieldError};

/// Optional documentation base for `error.doc_url` (relative path appended by callers).
pub const ERROR_DOC_BASE_URL: &str = "https://docs.proven.example/errors/";

/// Inner error object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<FieldError>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_url: Option<String>,
}

/// Standard API error response envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

impl ErrorResponse {
    pub fn from_app_error(err: &AppError, correlation_id: Option<String>) -> Self {
        let details = err.field_errors();
        Self {
            error: ErrorBody {
                code: err.error_code().to_string(),
                message: err.to_string(),
                details: if details.is_empty() {
                    None
                } else {
                    Some(details)
                },
                correlation_id,
                doc_url: Some(format!("{ERROR_DOC_BASE_URL}{}", err.error_code())),
            },
        }
    }
}

/// Legacy name kept for call sites that still say `ProblemDetails`.
/// Serializes as the nested `{ error: … }` envelope (ADR-0013).
pub type ProblemDetails = ErrorResponse;

impl From<&AppError> for ErrorResponse {
    fn from(err: &AppError) -> Self {
        Self::from_app_error(err, None)
    }
}
