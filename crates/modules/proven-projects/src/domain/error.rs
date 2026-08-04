//! Projects-specific error type, mapped to the platform [`AppError`] at the API edge.

use proven_shared::AppError;

#[derive(Debug, thiserror::Error)]
pub enum ProjectsError {
    #[error("{0} not found")]
    NotFound(&'static str),

    #[error("validation failed: {0}")]
    Validation(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl ProjectsError {
    pub fn not_found(resource: &'static str) -> Self {
        Self::NotFound(resource)
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict(message.into())
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Forbidden(message.into())
    }
}

impl From<ProjectsError> for AppError {
    fn from(err: ProjectsError) -> Self {
        match err {
            ProjectsError::NotFound(_) => AppError::NotFound,
            ProjectsError::Validation(msg) => AppError::Validation {
                message: msg,
                details: vec![],
            },
            ProjectsError::Conflict(msg) => AppError::Conflict(msg),
            ProjectsError::Forbidden(_) => AppError::Forbidden,
            ProjectsError::Internal(msg) => AppError::Internal(msg),
        }
    }
}

impl From<serde_json::Error> for ProjectsError {
    fn from(err: serde_json::Error) -> Self {
        ProjectsError::Internal(format!("serialization error: {err}"))
    }
}
