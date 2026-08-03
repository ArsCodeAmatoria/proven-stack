# Docker

Local development and image definitions for Proven.

## Quick start

```bash
./scripts/dev/up.sh
./scripts/dev/down.sh
```

Full service documentation: [Docker Local Development](../docs/engineering/DOCKER_LOCAL_DEVELOPMENT.md).

## Layout

| Path | Purpose |
| --- | --- |
| `compose/docker-compose.yml` | Full local stack (infra + api + worker + web) |
| `compose/docker-compose.deps.yml` | Infra only |
| `compose/docker-compose.ci.yml` | Minimal CI dependencies |
| `compose/temporal/dynamicconfig/` | Temporal local dynamic config |
| `Dockerfile.api` / `Dockerfile.api.dev` | Rust API (release / hot reload) |
| `Dockerfile.workers` / `Dockerfile.workers.dev` | Go worker (release / Air) |
| `Dockerfile.web.dev` | Next.js HMR |
| `air.notify-worker.toml` | Air config template for workers |

## Host ports

| Port | Service |
| --- | --- |
| 3000 | Next.js |
| 8080 | Rust API |
| 8088 | Temporal UI |
| 8091 | Go notify-worker |
| 5432 | PostgreSQL |
| 6379 | Redis |
| 4222 / 8222 | NATS client / monitor |
| 7233 | Temporal gRPC |
