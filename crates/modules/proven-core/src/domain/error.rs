//! Core-specific error type, mapped to the platform [`AppError`] at the API edge.

use proven_shared::AppError;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("{0} not found")]
    NotFound(&'static str),

    #[error("validation failed: {0}")]
    Validation(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("unauthorized")]
    Unauthorized,

    #[error("internal error: {0}")]
    Internal(String),
}

impl CoreError {
    pub fn not_found(resource: &'static str) -> Self {
        Self::NotFound(resource)
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict(message.into())
    }
}

impl From<CoreError> for AppError {
    fn from(err: CoreError) -> Self {
        match err {
            CoreError::NotFound(_) => AppError::NotFound,
            CoreError::Validation(msg) => AppError::Validation {
                message: msg,
                details: vec![],
            },
            CoreError::Conflict(msg) => AppError::Conflict(msg),
            CoreError::Forbidden(_) => AppError::Forbidden,
            CoreError::Unauthorized => AppError::Unauthorized,
            CoreError::Internal(msg) => AppError::Internal(msg),
        }
    }
}

impl From<sqlx::Error> for CoreError {
    fn from(err: sqlx::Error) -> Self {
        CoreError::Internal(format!("database error: {err}"))
    }
}

impl From<serde_json::Error> for CoreError {
    fn from(err: serde_json::Error) -> Self {
        CoreError::Internal(format!("serialization error: {err}"))
    }
}
