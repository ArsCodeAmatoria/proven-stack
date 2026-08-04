# proven-events

Shared NATS event library for Proven (ADR-0011).

## Features

- Subject naming: `proven.<module>.v<major>.<EventName>`
- Shared `EventEnvelope` + payload schema versioning
- `NatsEventPublisher` / `NatsEventSubscriber` with retry + tracing
- In-memory publishers/buses for tests
- Initial integration events: `CompanyCreated`, `UserCreated`, `ProjectCreated`,
  `AuditRecorded`, `FileUploaded`

## Quick start

```rust
use proven_events::{
    ActorRef, EventPublisher, InMemoryEventPublisher, InitialEvent, CompanyCreated,
};
use proven_shared::{CompanyId, TenantId};

# async fn demo() -> Result<(), proven_events::EventError> {
let publisher = InMemoryEventPublisher::new();
let envelope = InitialEvent::CompanyCreated(CompanyCreated {
    company_id: CompanyId::new(),
    legal_name: "Acme".into(),
    company_type: "prime".into(),
})
.into_envelope(TenantId::new(), ActorRef::System)?;
publisher.publish(envelope).await?;
# Ok(())
# }
```

## Docs

[NATS_EVENTS.md](../../docs/development/NATS_EVENTS.md) · [ADR-0011](../../docs/adr/0011-nats-event-system.md)
