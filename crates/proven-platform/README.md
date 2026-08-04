# Proven platform (`proven-platform`)

Axum composition root for the modular monolith. Registers **`proven-core`**, **`proven-companies`**, **`proven-users`**, and **`proven-projects`**. Event transport helpers come from **`proven-events`** (NATS publisher/subscriber builders on `AppState`). Temporal infrastructure comes from **`proven-temporal`** (`TemporalHandle` on `AppState`).

## Layout

```text
src/
  lib.rs
  state/           # AppState DI (infra + module handles)
  http/
    router.rs      # platform routes + merge(module routers)
    health.rs
    db.rs
    docs.rs        # /redoc
    error.rs
    middleware/
  infra/           # Postgres, Redis, NATS, Temporal, event pub/sub builders
  openapi.rs
  tracing_init.rs
```

## Endpoints

| Path | Notes |
| --- | --- |
| `GET /health`, `/healthz`, `/readyz` | Liveness / readiness |
| `GET /metrics` | Prometheus |
| `GET /api/v1/health`, `/api/v1/health/db`, `/api/v1/health/temporal`, `/api/v1/db/version` | Platform |
| `GET /docs`, `/redoc`, `/api-docs/openapi.json`, `/api/v1/openapi.json` | OpenAPI UIs / specs |
| `/api/v1/core/*` | Core HTTP |
| `/api/v1/companies/*` | Companies profile |
| `/api/v1/users/*` | Users account profile |
| `/api/v1/projects/*` | Projects Place skeleton |

## Events

When NATS is connected: `state.event_publisher()` / `state.event_subscriber()` — see
[NATS_EVENTS.md](../../docs/development/NATS_EVENTS.md).

## Boot

`AppState::connect` opens infra clients, applies migrations when migrate-on-start is set, and
installs in-memory module ports (Postgres schemas applied for cutover).
