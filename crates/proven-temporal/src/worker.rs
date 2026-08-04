//! Worker registration — binds registries to a task queue (no Temporal SDK poller yet).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::config::TemporalClientConfig;
use crate::error::TemporalError;
use crate::logging;
use crate::registry::{ActivityRegistry, WorkflowRegistry};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerStatus {
    pub task_queue: String,
    pub identity: String,
    pub running: bool,
    pub workflow_count: usize,
    pub activity_count: usize,
    pub workflows: Vec<String>,
    pub activities: Vec<String>,
}

/// Registers workflow/activity **definitions** for a future Temporal worker process.
///
/// In this infrastructure milestone the worker can be marked running for health/logging, but
/// it does not poll Temporal (SDK wiring is pending). Empty registries are valid.
pub struct WorkerRegistration {
    config: TemporalClientConfig,
    task_queue: String,
    workflows: WorkflowRegistry,
    activities: ActivityRegistry,
    running: AtomicBool,
}

impl WorkerRegistration {
    pub fn builder(config: TemporalClientConfig) -> WorkerBuilder {
        WorkerBuilder {
            config,
            task_queue: None,
            workflows: WorkflowRegistry::new(),
            activities: ActivityRegistry::new(),
        }
    }

    pub fn task_queue(&self) -> &str {
        &self.task_queue
    }

    pub fn workflows(&self) -> &WorkflowRegistry {
        &self.workflows
    }

    pub fn activities(&self) -> &ActivityRegistry {
        &self.activities
    }

    pub fn status(&self) -> WorkerStatus {
        WorkerStatus {
            task_queue: self.task_queue.clone(),
            identity: self.config.identity.clone(),
            running: self.running.load(Ordering::SeqCst),
            workflow_count: self.workflows.len(),
            activity_count: self.activities.len(),
            workflows: self.workflows.names(),
            activities: self.activities.names(),
        }
    }

    /// Mark the worker as running (infrastructure placeholder — no SDK poll loop).
    pub fn start(&self) -> Result<(), TemporalError> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Err(TemporalError::Worker("worker already running".into()));
        }
        let status = self.status();
        logging::log_worker_status(&status);
        tracing::info!(
            task_queue = %self.task_queue,
            workflows = status.workflow_count,
            activities = status.activity_count,
            "temporal worker registration started (infrastructure only; no poller)"
        );
        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        tracing::info!(task_queue = %self.task_queue, "temporal worker registration stopped");
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn into_shared(self) -> Arc<Self> {
        Arc::new(self)
    }
}

pub struct WorkerBuilder {
    config: TemporalClientConfig,
    task_queue: Option<String>,
    workflows: WorkflowRegistry,
    activities: ActivityRegistry,
}

impl WorkerBuilder {
    pub fn task_queue(mut self, queue: impl Into<String>) -> Self {
        self.task_queue = Some(queue.into());
        self
    }

    pub fn workflow_registry(mut self, registry: WorkflowRegistry) -> Self {
        self.workflows = registry;
        self
    }

    pub fn activity_registry(mut self, registry: ActivityRegistry) -> Self {
        self.activities = registry;
        self
    }

    pub fn register_workflow(
        mut self,
        def: crate::registry::WorkflowDefinition,
    ) -> Result<Self, TemporalError> {
        self.workflows.register(def)?;
        Ok(self)
    }

    pub fn register_activity(
        mut self,
        def: crate::registry::ActivityDefinition,
    ) -> Result<Self, TemporalError> {
        self.activities.register(def)?;
        Ok(self)
    }

    pub fn build(self) -> Result<WorkerRegistration, TemporalError> {
        self.config.validate()?;
        let task_queue = self
            .task_queue
            .unwrap_or_else(|| self.config.task_queues.domain.clone());
        if task_queue.trim().is_empty() {
            return Err(TemporalError::Config("task_queue must not be empty".into()));
        }
        Ok(WorkerRegistration {
            config: self.config,
            task_queue,
            workflows: self.workflows,
            activities: self.activities,
            running: AtomicBool::new(false),
        })
    }
}
