//! Workflow client port — start/signal/cancel/describe (infrastructure; no workflows yet).

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::config::TemporalClientConfig;
use crate::error::TemporalError;
use crate::health::{probe_tcp, TemporalHealthChecker};
use crate::logging;
use crate::registry::WorkflowRegistry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartWorkflowRequest {
    pub workflow_type: String,
    pub workflow_id: String,
    pub task_queue: String,
    pub input: Value,
    pub tenant_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartWorkflowResult {
    pub workflow_id: String,
    pub run_id: String,
    pub workflow_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowHandle {
    pub workflow_id: String,
    pub run_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescribeWorkflowResult {
    pub workflow_id: String,
    pub run_id: Option<String>,
    pub workflow_type: String,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
}

/// Module-facing Temporal port (RUST_BACKEND_ARCHITECTURE.md §21).
#[async_trait]
pub trait WorkflowClient: Send + Sync {
    async fn start_workflow(
        &self,
        req: StartWorkflowRequest,
    ) -> Result<StartWorkflowResult, TemporalError>;

    async fn signal_workflow(
        &self,
        handle: &WorkflowHandle,
        signal_name: &str,
        payload: Value,
    ) -> Result<(), TemporalError>;

    async fn cancel_workflow(&self, handle: &WorkflowHandle) -> Result<(), TemporalError>;

    async fn describe_workflow(
        &self,
        handle: &WorkflowHandle,
    ) -> Result<DescribeWorkflowResult, TemporalError>;

    fn workflow_registry(&self) -> &WorkflowRegistry;
}

/// Production-oriented client: TCP-probed connection settings + empty registry gate.
///
/// Full Temporal Rust SDK dial/start lands with `proven-workflows`. Until then,
/// `start_workflow` returns [`TemporalError::NoWorkflowsYet`] or
/// [`TemporalError::WorkflowNotRegistered`].
#[derive(Clone)]
pub struct TemporalWorkflowClient {
    config: TemporalClientConfig,
    workflows: Arc<WorkflowRegistry>,
    reachable: bool,
}

impl TemporalWorkflowClient {
    /// Probe Temporal and build a client with an empty workflow registry.
    pub async fn connect(config: TemporalClientConfig) -> Result<Self, TemporalError> {
        config.validate()?;
        probe_tcp(&config.address, config.connect_timeout_ms).await?;
        logging::log_client_connected(&config.address, &config.namespace);
        Ok(Self {
            config,
            workflows: Arc::new(WorkflowRegistry::new()),
            reachable: true,
        })
    }

    /// Build without probing (tests / optional infra).
    pub fn new_unchecked(config: TemporalClientConfig, workflows: WorkflowRegistry) -> Self {
        Self {
            config,
            workflows: Arc::new(workflows),
            reachable: false,
        }
    }

    pub fn config(&self) -> &TemporalClientConfig {
        &self.config
    }

    pub fn is_reachable(&self) -> bool {
        self.reachable
    }

    pub fn health_checker(&self) -> TemporalHealthChecker {
        TemporalHealthChecker::new(self.config.clone())
    }

    pub fn with_registry(mut self, registry: WorkflowRegistry) -> Self {
        self.workflows = Arc::new(registry);
        self
    }
}

#[async_trait]
impl WorkflowClient for TemporalWorkflowClient {
    async fn start_workflow(
        &self,
        req: StartWorkflowRequest,
    ) -> Result<StartWorkflowResult, TemporalError> {
        if req.workflow_type.trim().is_empty() {
            return Err(TemporalError::Validation(
                "workflow_type must not be empty".into(),
            ));
        }
        if req.workflow_id.trim().is_empty() {
            return Err(TemporalError::Validation(
                "workflow_id must not be empty".into(),
            ));
        }

        if self.workflows.is_empty() {
            logging::log_client_start_rejected(&req.workflow_type);
            return Err(TemporalError::NoWorkflowsYet);
        }

        if !self.workflows.contains(&req.workflow_type) {
            return Err(TemporalError::WorkflowNotRegistered(
                req.workflow_type.clone(),
            ));
        }

        // SDK start would go here once proven-workflows registers executables.
        Err(TemporalError::Internal(
            "Temporal SDK start is not wired yet — registry entry exists but executor is pending"
                .into(),
        ))
    }

    async fn signal_workflow(
        &self,
        handle: &WorkflowHandle,
        signal_name: &str,
        _payload: Value,
    ) -> Result<(), TemporalError> {
        if handle.workflow_id.trim().is_empty() {
            return Err(TemporalError::Validation(
                "workflow_id must not be empty".into(),
            ));
        }
        if signal_name.trim().is_empty() {
            return Err(TemporalError::Validation(
                "signal_name must not be empty".into(),
            ));
        }
        if self.workflows.is_empty() {
            return Err(TemporalError::NoWorkflowsYet);
        }
        Err(TemporalError::Internal(
            "Temporal SDK signal is not wired yet".into(),
        ))
    }

    async fn cancel_workflow(&self, handle: &WorkflowHandle) -> Result<(), TemporalError> {
        if handle.workflow_id.trim().is_empty() {
            return Err(TemporalError::Validation(
                "workflow_id must not be empty".into(),
            ));
        }
        if self.workflows.is_empty() {
            return Err(TemporalError::NoWorkflowsYet);
        }
        Err(TemporalError::Internal(
            "Temporal SDK cancel is not wired yet".into(),
        ))
    }

    async fn describe_workflow(
        &self,
        handle: &WorkflowHandle,
    ) -> Result<DescribeWorkflowResult, TemporalError> {
        if handle.workflow_id.trim().is_empty() {
            return Err(TemporalError::Validation(
                "workflow_id must not be empty".into(),
            ));
        }
        if self.workflows.is_empty() {
            return Err(TemporalError::NoWorkflowsYet);
        }
        Err(TemporalError::Internal(
            "Temporal SDK describe is not wired yet".into(),
        ))
    }

    fn workflow_registry(&self) -> &WorkflowRegistry {
        &self.workflows
    }
}

/// In-memory client for unit tests — tracks start attempts, never talks to Temporal.
#[derive(Default)]
pub struct InMemoryWorkflowClient {
    pub workflows: WorkflowRegistry,
    pub started: std::sync::Mutex<Vec<StartWorkflowRequest>>,
}

#[async_trait]
impl WorkflowClient for InMemoryWorkflowClient {
    async fn start_workflow(
        &self,
        req: StartWorkflowRequest,
    ) -> Result<StartWorkflowResult, TemporalError> {
        if self.workflows.is_empty() {
            return Err(TemporalError::NoWorkflowsYet);
        }
        if !self.workflows.contains(&req.workflow_type) {
            return Err(TemporalError::WorkflowNotRegistered(
                req.workflow_type.clone(),
            ));
        }
        self.started
            .lock()
            .map_err(|_| TemporalError::Internal("lock poisoned".into()))?
            .push(req.clone());
        Ok(StartWorkflowResult {
            workflow_id: req.workflow_id,
            run_id: Uuid::new_v4().to_string(),
            workflow_type: req.workflow_type,
        })
    }

    async fn signal_workflow(
        &self,
        _handle: &WorkflowHandle,
        _signal_name: &str,
        _payload: Value,
    ) -> Result<(), TemporalError> {
        Ok(())
    }

    async fn cancel_workflow(&self, _handle: &WorkflowHandle) -> Result<(), TemporalError> {
        Ok(())
    }

    async fn describe_workflow(
        &self,
        handle: &WorkflowHandle,
    ) -> Result<DescribeWorkflowResult, TemporalError> {
        Ok(DescribeWorkflowResult {
            workflow_id: handle.workflow_id.clone(),
            run_id: handle.run_id.clone(),
            workflow_type: "test".into(),
            status: "Running".into(),
            started_at: Some(Utc::now()),
        })
    }

    fn workflow_registry(&self) -> &WorkflowRegistry {
        &self.workflows
    }
}
