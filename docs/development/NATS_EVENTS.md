# NATS Event System

Canonical design: [ADR-0011](../adr/0011-nats-event-system.md) and
[EVENT_CATALOG.md](../architecture/EVENT_CATALOG.md).

Crate: `crates/proven-events`.

## Naming

```text
proven.<module>.v<major>.<EventName>
```

Examples:

- `proven.core.v1.CompanyCreated`
- `proven.core.v1.UserCreated`
- `proven.projects.v1.ProjectCreated`
- `proven.core.v1.AuditRecorded`
- `proven.core.v1.FileUploaded`

## Versioning

| Layer | Field | Rule |
| --- | --- | --- |
| Subject / name | `subject_major` (`v1` in subject) | Bump on breaking transport/name changes |
| Payload schema | `event_version` (`1.0.0`) | Additive within major; bump major on breaking payload |

Consumers must ignore unknown fields.

## Publisher / Subscriber

| Type | Role |
| --- | --- |
| `NatsEventPublisher` | Publish envelope JSON to NATS with retry + logging |
| `NatsEventSubscriber` | Subscribe to subject/wildcard; dispatch handlers with retry |
| `InMemoryEventPublisher` / `InMemoryEventBus` | Unit tests without NATS |

Platform helpers: `proven_platform::infra::event_publisher` / `event_subscriber`.

## Retry

`RetryPolicy` — exponential backoff with deterministic jitter (default 5 attempts).
Exhaustion → `EventError::RetryExhausted`.

## Logging

Publish and handle paths emit structured `tracing` spans/fields: `event_id`, `event_name`,
`subject`, `tenant_id`, attempt counts on failure.

## Initial events

| Event | Subject | Module |
| --- | --- | --- |
| `CompanyCreated` | `proven.core.v1.CompanyCreated` | core |
| `UserCreated` | `proven.core.v1.UserCreated` | core |
| `ProjectCreated` | `proven.projects.v1.ProjectCreated` | projects |
| `AuditRecorded` | `proven.core.v1.AuditRecorded` | core |
| `FileUploaded` | `proven.core.v1.FileUploaded` | core |

Build via `InitialEvent::…into_envelope(tenant, actor)` then `publisher.publish(envelope)`.

## Pending

- Transactional outbox publisher loop (`platform.outbox_messages` → NATS)
- JetStream durable consumers + DLQ (`proven.dlq.<module>.v1`)
- Automatic bridge from module domain outboxes to these integration events

## Tests

```bash
cargo test -p proven-events
```
