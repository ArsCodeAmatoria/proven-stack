# ADR-0011: NATS Event System

| Field | Value |
| --- | --- |
| Status | Accepted |
| Date | 2026-08-03 |
| Deciders | Lead Software Engineering |

## Context

Proven's integration bus is **NATS** ([EVENT_CATALOG.md](../architecture/EVENT_CATALOG.md)).
Module crates already raise typed domain events into in-memory outboxes, and the platform
connects an `async-nats` client for health — but there was no shared envelope library,
publisher/subscriber with retry, or initial catalog events.

## Decision

1. Add crate `crates/proven-events` — shared event library (not under `modules/`).
2. Subject naming: `proven.<module>.v<major>.<EventName>`.
3. Shared `EventEnvelope` with payload schema version (`event_version` semver) separate from
   subject major (`subject_major`).
4. Provide `NatsEventPublisher` / `NatsEventSubscriber` with exponential-backoff **retry** and
   structured **logging**; `InMemoryEventPublisher` / `InMemoryEventBus` for tests.
5. Ship initial integration events: `CompanyCreated`, `UserCreated`, `ProjectCreated`,
   `AuditRecorded`, `FileUploaded`.
6. Domain modules keep their own fine-grained events; adapters may map onto these integration
   shapes when publishing to NATS.
7. Transactional outbox → NATS relay remains a follow-up (table already exists); until then,
   callers may publish via `NatsEventPublisher` after commit or use the in-memory adapters in tests.

## Consequences

- `proven-platform` depends on `proven-events` and exposes publisher/subscriber builders.
- `proven-shared` still owns no events (kernel IDs only).
- JetStream durable consumers / DLQ are documented follow-ups.
