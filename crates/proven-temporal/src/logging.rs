//! Structured logging helpers for Temporal infrastructure.

use tracing::{debug, info, warn};

use crate::registry::{ActivityDefinition, WorkflowDefinition};
use crate::worker::WorkerStatus;

pub fn log_client_connected(address: &str, namespace: &str) {
    info!(
        address = %address,
        namespace = %namespace,
        "temporal client connected (infrastructure probe)"
    );
}

pub fn log_client_start_rejected(workflow_type: &str) {
    warn!(
        workflow_type = %workflow_type,
        "start_workflow rejected — no workflows registered yet (ADR-0012)"
    );
}

pub fn log_workflow_registered(def: &WorkflowDefinition) {
    debug!(
        workflow = %def.name,
        task_queue = %def.task_queue,
        "workflow metadata registered"
    );
}

pub fn log_activity_registered(def: &ActivityDefinition) {
    debug!(
        activity = %def.name,
        task_queue = %def.task_queue,
        "activity metadata registered"
    );
}

pub fn log_worker_status(status: &WorkerStatus) {
    info!(
        task_queue = %status.task_queue,
        running = status.running,
        workflows = status.workflow_count,
        activities = status.activity_count,
        "temporal worker status"
    );
}
