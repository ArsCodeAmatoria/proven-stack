//! Axum-friendly error → nested `{ error: … }` envelope (ADR-0013).

use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use proven_shared::{AppError, ErrorResponse};
use tracing::error;

/// Wrapper so handlers can return `Result<T, ApiError>`.
#[derive(Debug)]
pub struct ApiError {
    pub error: AppError,
    pub correlation_id: Option<String>,
}

impl ApiError {
    pub fn new(error: AppError) -> Self {
        Self {
            error,
            correlation_id: None,
        }
    }

    pub fn with_correlation(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }
}

impl From<AppError> for ApiError {
    fn from(value: AppError) -> Self {
        Self::new(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.error.status_code())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        if matches!(self.error, AppError::Internal(_)) {
            error!(error = %self.error, "internal API error");
        }

        let body = ErrorResponse::from_app_error(&self.error, self.correlation_id.clone());
        let mut response = (status, Json(body)).into_response();

        if let AppError::RateLimited {
            retry_after_secs, ..
        } = &self.error
        {
            if let Ok(value) = HeaderValue::from_str(&retry_after_secs.to_string()) {
                response.headers_mut().insert("retry-after", value);
            }
        }

        response
    }
}
