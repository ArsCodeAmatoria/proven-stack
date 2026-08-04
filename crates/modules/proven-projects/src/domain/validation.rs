//! Input validation helpers for Projects commands.

use crate::domain::ProjectsError;

pub fn require_non_empty(field: &str, value: &str) -> Result<(), ProjectsError> {
    if value.trim().is_empty() {
        return Err(ProjectsError::validation(format!("{field} must not be empty")));
    }
    Ok(())
}

pub fn require_code(code: &str) -> Result<(), ProjectsError> {
    require_non_empty("code", code)?;
    if code.len() > 64 {
        return Err(ProjectsError::validation("code must be at most 64 characters"));
    }
    Ok(())
}
