//! Axum-friendly error → HTTP problem details.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use proven_shared::{AppError, ProblemDetails};
use tracing::error;

/// Wrapper so handlers can return `Result<T, ApiError>`.
#[derive(Debug)]
pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
    fn from(value: AppError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self.0 {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        if matches!(self.0, AppError::Internal(_)) {
            error!(error = %self.0, "internal API error");
        }

        let body = ProblemDetails {
            title: status.canonical_reason().unwrap_or("Error").to_string(),
            status: status.as_u16(),
            detail: self.0.to_string(),
            code: self.0.error_code().to_string(),
        };

        (status, Json(body)).into_response()
    }
}
