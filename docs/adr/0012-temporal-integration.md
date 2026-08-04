# ADR-0012: Temporal Integration Infrastructure

| Field | Value |
| --- | --- |
| Status | Accepted |
| Date | 2026-08-03 |
| Deciders | Lead Software Engineering |

## Context

Proven orchestrates durable multi-step processes with **Temporal**
([TEMPORAL_WORKFLOWS.md](../architecture/TEMPORAL_WORKFLOWS.md)). The platform already probed
Temporal over TCP and Go workers dialed an empty SDK worker, but there was no shared Rust
client port, worker registration builder, workflow/activity metadata registries, retry
policy catalog, or dedicated health surface.

## Decision

1. Add crate `crates/proven-temporal` — Temporal **infrastructure only** (not under `modules/`).
2. Provide:
   - **Workflow client** port (`WorkflowClient`) + `TemporalWorkflowClient` / `InMemoryWorkflowClient`
   - **Worker registration** (`WorkerRegistration` / `WorkerBuilder`) bound to a task queue
   - **Workflow registry** and **Activity registry** (metadata; empty until workflows land)
   - **Retry policies** (`standard_activity_retry`, `standard_workflow_retry`, `io_activity_retry`)
   - **Error handling** (`TemporalError`, including `NoWorkflowsYet`)
   - **Logging** helpers for connect / register / reject / worker status
   - **Health checks** (TCP probe + registry counts)
3. **No workflows or activities** in this milestone. Client `start_workflow` returns
   `NoWorkflowsYet` while registries are empty.
4. Platform wires `TemporalHandle` via `proven-temporal` and exposes
   `GET /api/v1/health/temporal`.
5. Go `temporalx` keeps an empty SDK worker and explicit empty Workflow/Activity registries.
6. Future `proven-workflows` registers definitions/executors and wires the Temporal Rust SDK.

## Consequences

- Domain modules must not dial Temporal directly; they use the workflow client port.
- Empty registries are healthy infrastructure — readiness does not require workflow count > 0.
- Full SDK start/signal/cancel remains pending until `proven-workflows`.
