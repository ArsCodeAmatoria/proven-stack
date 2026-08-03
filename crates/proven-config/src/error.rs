use thiserror::Error;

/// Configuration load / validation failures. Safe to log — no secret values.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("missing required configuration: {keys}")]
    Missing { keys: String },

    #[error("invalid configuration for {key}: {reason}")]
    Invalid { key: String, reason: String },

    #[error("secrets validation failed: {reasons}")]
    Secrets { reasons: String },

    #[error("startup validation failed: {reasons}")]
    Startup { reasons: String },
}

impl ConfigError {
    pub fn missing(keys: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let mut list: Vec<String> = keys.into_iter().map(Into::into).collect();
        list.sort();
        list.dedup();
        Self::Missing {
            keys: list.join(", "),
        }
    }

    pub fn secrets(reasons: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let list: Vec<String> = reasons.into_iter().map(Into::into).collect();
        Self::Secrets {
            reasons: list.join("; "),
        }
    }

    pub fn startup(reasons: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let list: Vec<String> = reasons.into_iter().map(Into::into).collect();
        Self::Startup {
            reasons: list.join("; "),
        }
    }
}
