//! Activity definition registry — metadata only until Temporal SDK wiring.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::TemporalError;
use crate::retry::RetryPolicy;

/// Registered activity metadata (no executable body in this infrastructure milestone).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDefinition {
    pub name: String,
    pub task_queue: String,
    pub version: String,
    pub description: String,
    #[serde(skip)]
    pub retry: Option<RetryPolicy>,
}

impl ActivityDefinition {
    pub fn new(
        name: impl Into<String>,
        task_queue: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            task_queue: task_queue.into(),
            version: "1.0.0".into(),
            description: description.into(),
            retry: None,
        }
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    pub fn with_retry(mut self, retry: RetryPolicy) -> Self {
        self.retry = Some(retry);
        self
    }
}

/// Catalog of activity names that may be registered on a worker.
#[derive(Debug, Default, Clone)]
pub struct ActivityRegistry {
    entries: BTreeMap<String, ActivityDefinition>,
}

impl ActivityRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, def: ActivityDefinition) -> Result<(), TemporalError> {
        if def.name.trim().is_empty() {
            return Err(TemporalError::Validation(
                "activity name must not be empty".into(),
            ));
        }
        if self.entries.contains_key(&def.name) {
            return Err(TemporalError::Validation(format!(
                "activity '{}' is already registered",
                def.name
            )));
        }
        tracing::info!(
            activity = %def.name,
            task_queue = %def.task_queue,
            version = %def.version,
            "activity definition registered (metadata only)"
        );
        self.entries.insert(def.name.clone(), def);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&ActivityDefinition> {
        self.entries.get(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.entries.contains_key(name)
    }

    pub fn list(&self) -> Vec<&ActivityDefinition> {
        self.entries.values().collect()
    }

    pub fn names(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
