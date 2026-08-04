//! Temporal client / worker configuration.

use serde::{Deserialize, Serialize};

/// Well-known task queues (TEMPORAL_WORKFLOWS.md).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskQueues {
    /// Rust domain activities / workflows.
    pub domain: String,
    /// Go I/O worker queue.
    pub io: String,
    /// Go notify worker queue.
    pub notify: String,
    /// Go media worker queue.
    pub media: String,
    /// Go analytics worker queue.
    pub analytics: String,
}

impl Default for TaskQueues {
    fn default() -> Self {
        Self {
            domain: "proven-domain".into(),
            io: "proven-io".into(),
            notify: "proven-notify".into(),
            media: "proven-media".into(),
            analytics: "proven-analytics".into(),
        }
    }
}

/// Connection + queue settings for the Temporal infrastructure layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalClientConfig {
    pub address: String,
    pub namespace: String,
    pub identity: String,
    pub task_queues: TaskQueues,
    /// TCP probe timeout in milliseconds.
    pub connect_timeout_ms: u64,
}

impl TemporalClientConfig {
    pub fn new(address: impl Into<String>, namespace: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            namespace: namespace.into(),
            identity: "proven-api".into(),
            task_queues: TaskQueues::default(),
            connect_timeout_ms: 3_000,
        }
    }

    pub fn with_identity(mut self, identity: impl Into<String>) -> Self {
        self.identity = identity.into();
        self
    }

    pub fn validate(&self) -> Result<(), crate::error::TemporalError> {
        if self.address.trim().is_empty() {
            return Err(crate::error::TemporalError::Config(
                "address must not be empty".into(),
            ));
        }
        if self.namespace.trim().is_empty() {
            return Err(crate::error::TemporalError::Config(
                "namespace must not be empty".into(),
            ));
        }
        Ok(())
    }
}
