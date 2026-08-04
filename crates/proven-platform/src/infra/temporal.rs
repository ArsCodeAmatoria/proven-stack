//! Temporal client handle + infrastructure wiring (ADR-0012).
//!
//! Uses `proven-temporal` for the workflow client port, worker registration, registries,
//! retry policies, and health checks. Full Temporal Rust SDK activity/workflow executors
//! land with `proven-workflows` — registries stay empty here.

use proven_config::Config;
use proven_temporal::{
    TemporalClientConfig, TemporalHealth, TemporalHealthChecker, TemporalWorkflowClient,
    WorkerRegistration, WorkflowClient,
};
use tracing::info;

/// Process-local Temporal infrastructure handle for the API host.
#[derive(Clone)]
pub struct TemporalHandle {
    client: TemporalWorkflowClient,
    worker: std::sync::Arc<WorkerRegistration>,
}

impl TemporalHandle {
    pub fn address(&self) -> &str {
        &self.client.config().address
    }

    pub fn namespace(&self) -> &str {
        &self.client.config().namespace
    }

    pub fn client(&self) -> &TemporalWorkflowClient {
        &self.client
    }

    pub fn worker(&self) -> &WorkerRegistration {
        &self.worker
    }

    pub fn as_workflow_client(&self) -> &dyn WorkflowClient {
        &self.client
    }

    pub async fn health(&self) -> TemporalHealth {
        let checker = TemporalHealthChecker::new(self.client.config().clone());
        checker
            .check(self.worker.workflows(), self.worker.activities())
            .await
    }
}

pub async fn connect_temporal(config: &Config) -> anyhow::Result<TemporalHandle> {
    let client_config = TemporalClientConfig::new(
        config.temporal.address.clone(),
        config.temporal.namespace.clone(),
    )
    .with_identity(format!(
        "{}@{}",
        config.observability.service_name, config.temporal.namespace
    ));

    let client = TemporalWorkflowClient::connect(client_config.clone())
        .await
        .map_err(|err| anyhow::anyhow!(err))?;

    let worker = WorkerRegistration::builder(client_config)
        .task_queue(client.config().task_queues.domain.clone())
        .build()
        .map_err(|err| anyhow::anyhow!(err))?
        .into_shared();

    // Infrastructure-only: start registration bookkeeping (empty registries; no SDK poller).
    worker
        .start()
        .map_err(|err| anyhow::anyhow!(err))?;

    info!(
        address = %client.config().address,
        namespace = %client.config().namespace,
        task_queue = %worker.task_queue(),
        workflows = worker.workflows().len(),
        activities = worker.activities().len(),
        "temporal infrastructure ready (no workflows registered yet)"
    );

    Ok(TemporalHandle { client, worker })
}
