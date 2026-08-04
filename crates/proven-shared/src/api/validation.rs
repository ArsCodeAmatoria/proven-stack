//! Lightweight request validation helpers (wire-level; domain invariants stay in modules).

use uuid::Uuid;

use crate::error::{AppError, FieldError};

/// Accumulates field errors then converts to [`AppError::Validation`].
#[derive(Debug, Default)]
pub struct ValidationReport {
    details: Vec<FieldError>,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(
        &mut self,
        field: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.details.push(FieldError::new(field, code, message));
    }

    pub fn is_ok(&self) -> bool {
        self.details.is_empty()
    }

    pub fn finish(self) -> Result<(), AppError> {
        if self.details.is_empty() {
            Ok(())
        } else {
            Err(AppError::Validation {
                message: "validation failed".into(),
                details: self.details,
            })
        }
    }

    pub fn into_details(self) -> Vec<FieldError> {
        self.details
    }
}

pub fn require_non_empty(value: &str, field: &str) -> Result<(), AppError> {
    if value.trim().is_empty() {
        Err(AppError::Validation {
            message: format!("{field} is required"),
            details: vec![FieldError::new(field, "required", "must not be empty")],
        })
    } else {
        Ok(())
    }
}

pub fn require_uuid(raw: &str, field: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(raw.trim()).map_err(|_| AppError::Validation {
        message: format!("{field} must be a UUID"),
        details: vec![FieldError::new(field, "invalid_uuid", "must be a UUID")],
    })
}
