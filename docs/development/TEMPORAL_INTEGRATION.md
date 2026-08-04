# Temporal Integration

Canonical design: [ADR-0012](../adr/0012-temporal-integration.md) and
[TEMPORAL_WORKFLOWS.md](../architecture/TEMPORAL_WORKFLOWS.md).

Crate: `crates/proven-temporal` (infrastructure only — **no workflows yet**).

## Components

| Piece | Role |
| --- | --- |
| `WorkflowClient` | Port: start / signal / cancel / describe |
| `TemporalWorkflowClient` | Production client (TCP probe + registry gate) |
| `InMemoryWorkflowClient` | Unit tests without Temporal |
| `WorkerRegistration` | Task-queue binding + registry bookkeeping (no SDK poller yet) |
| `WorkflowRegistry` / `ActivityRegistry` | Metadata catalogs (start empty) |
| `RetryPolicy` | Defaults for future activities/workflows |
| `TemporalError` | Typed errors (`NoWorkflowsYet`, not registered, connection, …) |
| `TemporalHealthChecker` | TCP reachability + registry counts |

## Task queues

| Queue | Host |
| --- | --- |
| `proven-domain` | Rust domain activities/workflows |
| `proven-io` | Go I/O |
| `proven-notify` | Go notify |
| `proven-media` | Go media |
| `proven-analytics` | Go analytics |

## Retry defaults

| Policy | Max attempts | Notes |
| --- | --- | --- |
| `standard_activity_retry` | 5 | Non-retry: ValidationError, Forbidden, NotFound, Conflict, BadRequest |
| `standard_workflow_retry` | 3 | Conservative workflow-level |
| `io_activity_retry` | 8 | Transient provider failures (Go) |

## Platform wiring

- `proven_platform::infra::connect_temporal` builds `TemporalHandle` (client + empty worker registration).
- `AppState::temporal()` exposes the handle.
- Health: `GET /api/v1/health/temporal`.

## Go parity

`go/internal/platform/temporalx` dials the Temporal SDK, starts an empty worker, and exposes
`WorkflowRegistry` / `ActivityRegistry` (empty by default).

## Pending

- `proven-workflows` module: register real workflow/activity executors
- Temporal Rust SDK dial for start/signal/cancel/describe
- Worker poll loop on Rust domain queue
- Workflow instance projection APIs (`workflows.workflow_instances`)

## Tests

```bash
cargo test -p proven-temporal
cargo test -p proven-platform
cd go && go test ./internal/platform/temporalx/...
```
