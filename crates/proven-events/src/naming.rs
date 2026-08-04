//! Subject naming: `proven.<module>.v<major>.<EventName>` (EVENT_CATALOG.md §2.1).

use crate::error::EventError;

/// Parsed NATS subject parts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectParts {
    pub module: String,
    pub major: u32,
    pub event_name: String,
}

impl SubjectParts {
    pub fn subject(&self) -> String {
        format!("proven.{}.v{}.{}", self.module, self.major, self.event_name)
    }
}

/// Build a canonical subject string.
pub fn event_subject(module: &str, major: u32, event_name: &str) -> String {
    SubjectParts {
        module: module.to_string(),
        major,
        event_name: event_name.to_string(),
    }
    .subject()
}

/// Parse `proven.<module>.v<major>.<EventName>`.
pub fn parse_subject(subject: &str) -> Result<SubjectParts, EventError> {
    let parts: Vec<&str> = subject.split('.').collect();
    if parts.len() != 4 {
        return Err(EventError::InvalidSubject(format!(
            "expected proven.<module>.v<major>.<EventName>, got '{subject}'"
        )));
    }
    if parts[0] != "proven" {
        return Err(EventError::InvalidSubject(format!(
            "subject must start with 'proven', got '{subject}'"
        )));
    }
    let module = parts[1].to_string();
    if module.is_empty() {
        return Err(EventError::InvalidSubject("empty module".into()));
    }
    let version = parts[2];
    if !version.starts_with('v') {
        return Err(EventError::InvalidSubject(format!(
            "version segment must look like v1, got '{version}'"
        )));
    }
    let major: u32 = version[1..].parse().map_err(|_| {
        EventError::InvalidSubject(format!("invalid major version in '{version}'"))
    })?;
    let event_name = parts[3].to_string();
    if event_name.is_empty() {
        return Err(EventError::InvalidSubject("empty event name".into()));
    }
    Ok(SubjectParts {
        module,
        major,
        event_name,
    })
}

/// Typed subject helper bound to an event name + module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventSubject {
    pub module: &'static str,
    pub major: u32,
    pub event_name: &'static str,
}

impl EventSubject {
    pub const fn new(module: &'static str, major: u32, event_name: &'static str) -> Self {
        Self {
            module,
            major,
            event_name,
        }
    }

    pub fn as_str(&self) -> String {
        event_subject(self.module, self.major, self.event_name)
    }
}

/// Wildcard for a module stream (all events under a major version).
pub fn module_wildcard(module: &str, major: u32) -> String {
    format!("proven.{module}.v{major}.*")
}
