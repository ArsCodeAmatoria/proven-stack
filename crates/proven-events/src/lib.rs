//! Proven shared event library — transport-agnostic envelope + NATS publisher/subscriber.
//!
//! Domain modules still own their fine-grained domain events. This crate owns:
//! - subject naming (`proven.<module>.v<major>.<EventName>`)
//! - shared envelope + versioning helpers
//! - initial integration events (`CompanyCreated`, `UserCreated`, …)
//! - NATS publisher / subscriber with retry + structured logging
//!
//! See [ADR-0011](../../docs/adr/0011-nats-event-system.md) and
//! [`docs/development/NATS_EVENTS.md`](../../docs/development/NATS_EVENTS.md).

pub mod envelope;
pub mod error;
pub mod events;
pub mod naming;
pub mod publisher;
pub mod retry;
pub mod subscriber;

pub use envelope::{ActorRef, EventEnvelope, ResourceRef};
pub use error::EventError;
pub use events::{
    AuditRecorded, CompanyCreated, FileUploaded, InitialEvent, ProjectCreated, UserCreated,
};
pub use naming::{event_subject, module_wildcard, parse_subject, EventSubject, SubjectParts};
pub use publisher::{
    EventPublisher, InMemoryEventPublisher, NatsEventPublisher, PublishOptions,
};
pub use retry::{retry_with_backoff, RetryPolicy};
pub use subscriber::{
    EventHandler, FnHandler, InMemoryEventBus, NatsEventSubscriber, SubscribeOptions,
    SubscriptionHandle,
};
