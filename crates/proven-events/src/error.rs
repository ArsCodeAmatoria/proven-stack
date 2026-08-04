//! Shared event errors.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EventError {
    #[error("invalid subject: {0}")]
    InvalidSubject(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("publish failed: {0}")]
    Publish(String),

    #[error("subscribe failed: {0}")]
    Subscribe(String),

    #[error("handler failed: {0}")]
    Handler(String),

    #[error("retry exhausted after {attempts} attempts: {message}")]
    RetryExhausted { attempts: u32, message: String },

    #[error("internal error: {0}")]
    Internal(String),
}

impl From<serde_json::Error> for EventError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}
