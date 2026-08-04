//! Temporal infrastructure tests (ADR-0012) — no live Temporal required.

use std::time::Duration;

use proven_temporal::{
    io_activity_retry, standard_activity_retry, standard_workflow_retry, ActivityDefinition,
    ActivityRegistry, InMemoryWorkflowClient, StartWorkflowRequest, TemporalClientConfig,
    TemporalError, TemporalHealthChecker, TemporalHealthStatus, WorkerRegistration,
    WorkflowClient, WorkflowDefinition, WorkflowRegistry,
};
use serde_json::json;
use uuid::Uuid;

#[test]
fn retry_policies_backoff_and_non_retryable() {
    let activity = standard_activity_retry();
    assert_eq!(activity.max_attempts, 5);
    assert!(!activity.is_retryable("ValidationError"));
    assert!(activity.is_retryable("Unavailable"));
    assert!(activity.interval_for_attempt(1) <= Duration::from_secs(1));
    assert!(activity.interval_for_attempt(3) > activity.interval_for_attempt(1));

    let workflow = standard_workflow_retry();
    assert_eq!(workflow.max_attempts, 3);

    let io = io_activity_retry();
    assert_eq!(io.max_attempts, 8);
}

#[test]
fn workflow_and_activity_registries() {
    let mut workflows = WorkflowRegistry::new();
    workflows
        .register(
            WorkflowDefinition::new("ExampleWorkflow", "proven-domain", "example")
                .with_retry(standard_workflow_retry()),
        )
        .expect("register workflow");
    assert!(workflows.contains("ExampleWorkflow"));
    assert_eq!(workflows.len(), 1);
    assert!(workflows
        .register(WorkflowDefinition::new("ExampleWorkflow", "proven-domain", "dup"))
        .is_err());

    let mut activities = ActivityRegistry::new();
    activities
        .register(
            ActivityDefinition::new("DoThing", "proven-domain", "example activity")
                .with_retry(standard_activity_retry()),
        )
        .expect("register activity");
    assert_eq!(activities.names(), vec!["DoThing".to_string()]);
}

#[tokio::test]
async fn worker_registration_empty_is_valid() {
    let config = TemporalClientConfig::new("127.0.0.1:7233", "default");
    let worker = WorkerRegistration::builder(config)
        .task_queue("proven-domain")
        .build()
        .expect("build worker");

    assert!(worker.workflows().is_empty());
    assert!(worker.activities().is_empty());
    worker.start().expect("start");
    assert!(worker.is_running());
    let status = worker.status();
    assert_eq!(status.task_queue, "proven-domain");
    assert_eq!(status.workflow_count, 0);
    worker.stop();
    assert!(!worker.is_running());
}

#[tokio::test]
async fn workflow_client_rejects_start_without_registry() {
    let client = InMemoryWorkflowClient::default();
    let err = client
        .start_workflow(StartWorkflowRequest {
            workflow_type: "Anything".into(),
            workflow_id: "t:w:1".into(),
            task_queue: "proven-domain".into(),
            input: json!({}),
            tenant_id: Some(Uuid::new_v4()),
        })
        .await
        .expect_err("must reject");
    assert_eq!(err, TemporalError::NoWorkflowsYet);
}

#[tokio::test]
async fn in_memory_client_starts_when_registered() {
    let mut workflows = WorkflowRegistry::new();
    workflows
        .register(WorkflowDefinition::new(
            "DemoWorkflow",
            "proven-domain",
            "demo",
        ))
        .unwrap();
    let client = InMemoryWorkflowClient {
        workflows,
        started: Default::default(),
    };

    let result = client
        .start_workflow(StartWorkflowRequest {
            workflow_type: "DemoWorkflow".into(),
            workflow_id: "tenant:DemoWorkflow:x:1".into(),
            task_queue: "proven-domain".into(),
            input: json!({"ok": true}),
            tenant_id: None,
        })
        .await
        .expect("start");
    assert_eq!(result.workflow_type, "DemoWorkflow");
    assert!(!result.run_id.is_empty());
}

#[tokio::test]
async fn health_checker_reports_unreachable_for_bad_address() {
    let config = TemporalClientConfig::new("127.0.0.1:1", "default");
    let checker = TemporalHealthChecker::new(config);
    let health = checker
        .check(&WorkflowRegistry::new(), &ActivityRegistry::new())
        .await;
    assert_eq!(health.status, TemporalHealthStatus::Unavailable);
    assert!(!health.reachable);
}

#[test]
fn config_validation() {
    let bad = TemporalClientConfig::new("", "default");
    assert!(bad.validate().is_err());
    let ok = TemporalClientConfig::new("127.0.0.1:7233", "default");
    assert!(ok.validate().is_ok());
    assert_eq!(ok.task_queues.domain, "proven-domain");
}
