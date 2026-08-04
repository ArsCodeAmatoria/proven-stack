//! Retry policies for Temporal workflows and activities (infrastructure defaults).

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Retry policy used when workflows/activities are eventually registered.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetryPolicy {
    pub name: String,
    pub max_attempts: u32,
    pub initial_interval: Duration,
    pub backoff_coefficient: f64,
    pub max_interval: Duration,
    /// Non-retryable error type names (domain 4xx equivalents).
    pub non_retryable_error_types: Vec<String>,
}

impl RetryPolicy {
    pub fn interval_for_attempt(&self, attempt: u32) -> Duration {
        if attempt <= 1 {
            return self.initial_interval;
        }
        let exp = self.backoff_coefficient.powi((attempt - 1) as i32);
        let ms = (self.initial_interval.as_millis() as f64 * exp) as u64;
        Duration::from_millis(ms.min(self.max_interval.as_millis() as u64))
    }

    pub fn is_retryable(&self, error_type: &str) -> bool {
        !self
            .non_retryable_error_types
            .iter()
            .any(|t| t == error_type)
    }
}

/// Error type names that must not be retried for domain activities.
pub const STANDARD_ACTIVITY_NON_RETRYABLE: &[&str] = &[
    "ValidationError",
    "Forbidden",
    "NotFound",
    "Conflict",
    "BadRequest",
];

/// Domain activity defaults: few attempts, no retry on validation/forbidden.
pub fn standard_activity_retry() -> RetryPolicy {
    RetryPolicy {
        name: "standard_activity".into(),
        max_attempts: 5,
        initial_interval: Duration::from_secs(1),
        backoff_coefficient: 2.0,
        max_interval: Duration::from_secs(60),
        non_retryable_error_types: STANDARD_ACTIVITY_NON_RETRYABLE
            .iter()
            .map(|s| (*s).to_string())
            .collect(),
    }
}

/// Workflow-level retry (usually rely on activity retries; keep conservative).
pub fn standard_workflow_retry() -> RetryPolicy {
    RetryPolicy {
        name: "standard_workflow".into(),
        max_attempts: 3,
        initial_interval: Duration::from_secs(2),
        backoff_coefficient: 2.0,
        max_interval: Duration::from_secs(30),
        non_retryable_error_types: vec!["ValidationError".into(), "Cancelled".into()],
    }
}

/// I/O activity defaults (Go workers) — more attempts for transient provider failures.
pub fn io_activity_retry() -> RetryPolicy {
    RetryPolicy {
        name: "io_activity".into(),
        max_attempts: 8,
        initial_interval: Duration::from_secs(2),
        backoff_coefficient: 2.0,
        max_interval: Duration::from_secs(120),
        non_retryable_error_types: vec![
            "ValidationError".into(),
            "Forbidden".into(),
            "NotFound".into(),
        ],
    }
}
