//! Temporal infrastructure errors.

use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TemporalError {
    #[error("temporal connection failed: {0}")]
    Connection(String),

    #[error("temporal not ready: {0}")]
    NotReady(String),

    #[error("workflow '{0}' is not registered")]
    WorkflowNotRegistered(String),

    #[error("activity '{0}' is not registered")]
    ActivityNotRegistered(String),

    #[error("no workflows registered — infrastructure only (ADR-0012)")]
    NoWorkflowsYet,

    #[error("worker error: {0}")]
    Worker(String),

    #[error("invalid configuration: {0}")]
    Config(String),

    #[error("validation failed: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),
}
