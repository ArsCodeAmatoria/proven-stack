//! Unit tests for naming, versioning, retry, publisher/subscriber (in-memory).

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use proven_events::events::subjects;
use proven_events::{
    event_subject, module_wildcard, parse_subject, retry_with_backoff, ActorRef, AuditRecorded,
    CompanyCreated, EventEnvelope, EventError, EventHandler, EventPublisher, FileUploaded,
    InMemoryEventBus, InMemoryEventPublisher, InitialEvent, ProjectCreated, RetryPolicy,
    SubscribeOptions, UserCreated,
};
use proven_shared::{CompanyId, FileObjectId, ProjectId, TenantId, UserId};
use tokio::sync::Notify;

struct CountingHandler {
    count: AtomicU32,
    fail_times: AtomicU32,
    notify: Notify,
}

#[async_trait]
impl EventHandler for CountingHandler {
    async fn handle(&self, envelope: EventEnvelope) -> Result<(), EventError> {
        let remaining = self.fail_times.load(Ordering::SeqCst);
        if remaining > 0 {
            self.fail_times.fetch_sub(1, Ordering::SeqCst);
            return Err(EventError::Handler("transient".into()));
        }
        assert_eq!(envelope.event_name, "ProjectCreated");
        self.count.fetch_add(1, Ordering::SeqCst);
        self.notify.notify_waiters();
        Ok(())
    }
}

#[test]
fn subject_naming_round_trip() {
    let subject = event_subject("projects", 1, "ProjectCreated");
    assert_eq!(subject, "proven.projects.v1.ProjectCreated");
    let parts = parse_subject(&subject).expect("parse");
    assert_eq!(parts.module, "projects");
    assert_eq!(parts.major, 1);
    assert_eq!(parts.event_name, "ProjectCreated");
    assert_eq!(module_wildcard("core", 1), "proven.core.v1.*");
}

#[test]
fn subject_constants_match_catalog() {
    assert_eq!(
        subjects::COMPANY_CREATED.as_str(),
        "proven.core.v1.CompanyCreated"
    );
    assert_eq!(subjects::USER_CREATED.as_str(), "proven.core.v1.UserCreated");
    assert_eq!(
        subjects::PROJECT_CREATED.as_str(),
        "proven.projects.v1.ProjectCreated"
    );
    assert_eq!(
        subjects::AUDIT_RECORDED.as_str(),
        "proven.core.v1.AuditRecorded"
    );
    assert_eq!(
        subjects::FILE_UPLOADED.as_str(),
        "proven.core.v1.FileUploaded"
    );
}

#[test]
fn reject_invalid_subjects() {
    assert!(parse_subject("bad").is_err());
    assert!(parse_subject("foo.core.v1.X").is_err());
    assert!(parse_subject("proven.core.1.X").is_err());
}

#[test]
fn envelope_versioning_helpers() {
    let tenant = TenantId::new();
    let envelope = EventEnvelope::new(
        "core",
        "UserCreated",
        tenant,
        ActorRef::System,
        proven_events::ResourceRef {
            resource_type: "user".into(),
            resource_id: UserId::new().as_uuid(),
        },
        serde_json::json!({}),
    )
    .with_event_version("2.1.0")
    .with_subject_major(2);

    assert_eq!(envelope.payload_major(), 2);
    assert_eq!(envelope.subject(), "proven.core.v2.UserCreated");
    assert_eq!(envelope.event_version, "2.1.0");
}

#[tokio::test]
async fn initial_events_publish_in_memory() {
    let publisher = InMemoryEventPublisher::new();
    let tenant = TenantId::new();

    let events = vec![
        InitialEvent::CompanyCreated(CompanyCreated {
            company_id: CompanyId::new(),
            legal_name: "Acme".into(),
            company_type: "prime".into(),
        }),
        InitialEvent::UserCreated(UserCreated {
            user_id: UserId::new(),
            email: "a@example.com".into(),
            display_name: Some("A".into()),
        }),
        InitialEvent::ProjectCreated(ProjectCreated {
            project_id: ProjectId::new(),
            code: "P-1".into(),
            name: "Site".into(),
            prime_contractor_company_id: CompanyId::new(),
        }),
        InitialEvent::AuditRecorded(AuditRecorded {
            audit_entry_id: uuid::Uuid::new_v4(),
            action: "core.file.completed".into(),
            resource_type: "file_object".into(),
            resource_id: Some(uuid::Uuid::new_v4()),
            project_id: None,
        }),
        InitialEvent::FileUploaded(FileUploaded {
            file_id: FileObjectId::new(),
            object_class: "pdf".into(),
            content_type: Some("application/pdf".into()),
            byte_size: Some(12),
            storage_key: "t/pdfs/x".into(),
        }),
    ];

    for event in events {
        let envelope = event
            .into_envelope(tenant, ActorRef::System)
            .expect("envelope");
        publisher.publish(envelope).await.expect("publish");
    }

    let published = publisher.published().expect("list");
    assert_eq!(published.len(), 5);
    assert!(published.iter().all(|e| e.published_at.is_some()));
    assert_eq!(published[0].subject(), "proven.core.v1.CompanyCreated");
    assert_eq!(published[2].subject(), "proven.projects.v1.ProjectCreated");
}

#[tokio::test]
async fn in_memory_bus_pub_sub_with_retry() {
    let bus = Arc::new(InMemoryEventBus::new(32));
    let handler = Arc::new(CountingHandler {
        count: AtomicU32::new(0),
        fail_times: AtomicU32::new(2),
        notify: Notify::new(),
    });

    let options = SubscribeOptions {
        retry: RetryPolicy {
            max_attempts: 5,
            initial_backoff: Duration::from_millis(1),
            max_backoff: Duration::from_millis(5),
            multiplier: 2.0,
        },
        ack_on_handler_failure: true,
    };

    let _sub = bus
        .subscribe(
            "proven.projects.v1.*",
            FnHandlerAdapter(handler.clone()),
            options,
        )
        .await;

    let envelope = InitialEvent::ProjectCreated(ProjectCreated {
        project_id: ProjectId::new(),
        code: "X".into(),
        name: "Y".into(),
        prime_contractor_company_id: CompanyId::new(),
    })
    .into_envelope(TenantId::new(), ActorRef::System)
    .expect("env");

    bus.publish(envelope).await.expect("publish");

    tokio::time::timeout(Duration::from_secs(2), handler.notify.notified())
        .await
        .expect("handler should succeed after retries");
    assert_eq!(handler.count.load(Ordering::SeqCst), 1);
}

/// Local adapter so we can pass Arc<CountingHandler> without orphan impl issues.
struct FnHandlerAdapter(Arc<CountingHandler>);

#[async_trait]
impl EventHandler for FnHandlerAdapter {
    async fn handle(&self, envelope: EventEnvelope) -> Result<(), EventError> {
        self.0.handle(envelope).await
    }
}

#[tokio::test]
async fn retry_exhausted() {
    let policy = RetryPolicy {
        max_attempts: 3,
        initial_backoff: Duration::from_millis(1),
        max_backoff: Duration::from_millis(2),
        multiplier: 2.0,
    };
    let result = retry_with_backoff(&policy, "always_fail", |_n| async {
        Err::<(), _>(EventError::Publish("nope".into()))
    })
    .await;
    match result {
        Err(EventError::RetryExhausted { attempts, .. }) => assert_eq!(attempts, 3),
        other => panic!("expected RetryExhausted, got {other:?}"),
    }
}
