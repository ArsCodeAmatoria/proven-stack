//! Proven Temporal infrastructure (ADR-0012).
//!
//! Provides the workflow **client port**, **worker registration**, **workflow/activity
//! registries**, **retry policies**, **error handling**, **logging**, and **health checks**.
//!
//! **No workflows or activities are registered yet** — registries start empty and the client
//! refuses to start runs until a future `proven-workflows` module registers definitions.
//!
//! See [`docs/development/TEMPORAL_INTEGRATION.md`](../../docs/development/TEMPORAL_INTEGRATION.md).

pub mod client;
pub mod config;
pub mod error;
pub mod health;
pub mod logging;
pub mod registry;
pub mod retry;
pub mod worker;

pub use client::{
    DescribeWorkflowResult, InMemoryWorkflowClient, StartWorkflowRequest, StartWorkflowResult,
    TemporalWorkflowClient, WorkflowClient, WorkflowHandle,
};
pub use config::{TaskQueues, TemporalClientConfig};
pub use error::TemporalError;
pub use health::{TemporalHealth, TemporalHealthChecker, TemporalHealthStatus};
pub use registry::{
    ActivityDefinition, ActivityRegistry, WorkflowDefinition, WorkflowRegistry,
};
pub use retry::{
    io_activity_retry, standard_activity_retry, standard_workflow_retry, RetryPolicy,
    STANDARD_ACTIVITY_NON_RETRYABLE,
};
pub use worker::{WorkerBuilder, WorkerRegistration, WorkerStatus};
