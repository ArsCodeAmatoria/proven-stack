# Proven platform (`proven-platform`)

Axum host for the modular monolith. **No domain modules** are registered yet.

## Layout

```text
src/
  lib.rs
  state/           # AppState DI container
  http/
    router.rs      # route composition
    health.rs      # /health, /healthz, /readyz, /api/v1/health
    db.rs          # /api/v1/health/db, /api/v1/db/version
    error.rs       # AppError → HTTP problem details
    middleware/    # request-id + tracing
  infra/
    db.rs          # SQLx Postgres pool (via proven-db)
    redis.rs       # Redis connection manager
    nats.rs        # async-nats client
    temporal.rs    # Temporal address handle + TCP probe
  openapi.rs       # utoipa OpenAPI + /docs Swagger UI
  tracing_init.rs  # logging / tracing subscriber
```

## Endpoints

| Path | Status | Notes |
| --- | --- | --- |
| `GET /health` | **200** | Liveness |
| `GET /healthz` | 200 | Alias |
| `GET /readyz` | 200/503 | Live Postgres probe + other infra |
| `GET /metrics` | 200/404 | Prometheus scrape (when enabled) |
| `GET /api/v1/health` | 200 | Version envelope |
| `GET /api/v1/health/db` | 200/503 | Pool `SELECT 1` latency |
| `GET /api/v1/db/version` | 200/503 | Postgres + migration metadata |
| `GET /docs` | 200 | Swagger UI |
| `GET /api-docs/openapi.json` | 200 | OpenAPI JSON |

## Observability

Structured logs (`proven-observability`), `x-request-id` / `x-correlation-id`, optional OTLP traces, Prometheus `/metrics`. No dashboards in this milestone.

## Boot

`AppState::connect` opens Postgres, Redis, NATS, and Temporal. When `PROVEN_MIGRATE_ON_START=true`, pending sqlx migrations under `PROVEN_MIGRATIONS_DIR` run after the pool connects. When `PROVEN_INFRA_OPTIONAL=true` (default in development), connection failures warn and the API still serves `/health`.
