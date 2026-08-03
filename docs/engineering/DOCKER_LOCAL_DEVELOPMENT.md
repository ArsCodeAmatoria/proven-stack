# Proven — Docker Local Development

| Field | Value |
| --- | --- |
| **Document type** | Engineering guide |
| **Status** | Active |
| **Last updated** | 2026-08-03 |
| **Compose file** | [`docker/compose/docker-compose.yml`](../../docker/compose/docker-compose.yml) |
| **Companion** | [Development Guide](./DEVELOPMENT.md), [Deployment Architecture](../architecture/DEPLOYMENT_ARCHITECTURE.md) |

---

## 1. Purpose

Run the **full local development stack** with Docker Compose:

- Infrastructure: PostgreSQL, Redis, NATS, Temporal, Temporal UI  
- Applications: Rust API, Go worker, Next.js web  

No business logic is included. App containers expose health endpoints and support **hot reload** where practical.

---

## 2. Prerequisites

| Tool | Notes |
| --- | --- |
| **Docker** Desktop / Engine | 24+ recommended |
| **Docker Compose** | v2 (`docker compose`) |
| **Git** | Clone of `proven-stack` |

Host Node/Rust/Go are **not** required when using the full Compose stack.

---

## 3. Quick start

```bash
cp .env.example .env          # once
./scripts/dev/up.sh           # build + start (detached)
./scripts/dev/ps.sh           # status
./scripts/dev/logs.sh         # follow logs (Ctrl+C to detach viewer)
./scripts/dev/down.sh         # stop
```

First boot builds images and compiles the Rust API inside the container — allow several minutes.

| Make target | Equivalent |
| --- | --- |
| `make docker-up` | `./scripts/dev/up.sh` |
| `make docker-down` | `./scripts/dev/down.sh` |
| `make docker-logs` | `./scripts/dev/logs.sh` |
| `make docker-ps` | `./scripts/dev/ps.sh` |
| `make docker-deps` | `./scripts/dev/up.sh --deps-only` |

---

## 4. Service catalog

### 4.1 Summary

| Service | Container | Host port(s) | Hot reload | Purpose |
| --- | --- | --- | --- | --- |
| **postgres** | `postgres` | `5432` | n/a | Primary SQL store (app + Temporal schemas) |
| **redis** | `redis` | `6379` | n/a | Cache / ephemeral coordination (AOF on) |
| **nats** | `nats` | `4222`, `8222` | n/a | Messaging + JetStream; monitor on `8222` |
| **temporal** | `temporal` | `7233` | n/a | Workflow engine (auto-setup image) |
| **temporal-ui** | `temporal-ui` | `8088` → `8080` | n/a | Temporal web UI |
| **api** | `api` | `8080` | **Yes** (`cargo-watch`) | Rust Axum host (`proven-api`) |
| **worker** | `worker` | `8091` | **Yes** (Air) | Go `notify-worker` (foundation health server) |
| **web** | `web` | `3000` | **Yes** (Next.js HMR) | Next.js App Router shell |

### 4.2 PostgreSQL (`postgres`)

| | |
| --- | --- |
| **Image** | `postgres:16-alpine` |
| **Credentials** | user/password/db: `proven` / `proven` / `proven` |
| **URL (host)** | `postgres://proven:proven@127.0.0.1:5432/proven` |
| **URL (Compose)** | `postgres://proven:proven@postgres:5432/proven` |
| **Volume** | `proven_pg` |
| **Health** | `pg_isready` |

Temporal’s auto-setup process creates additional databases/schemas on this same instance (`temporal`, `temporal_visibility`). App domain migrations are not applied in foundation.

### 4.3 Redis (`redis`)

| | |
| --- | --- |
| **Image** | `redis:7-alpine` |
| **URL (host)** | `redis://127.0.0.1:6379` |
| **URL (Compose)** | `redis://redis:6379` |
| **Persistence** | AOF (`--appendonly yes`), volume `proven_redis` |
| **Health** | `PING` |

Used later for cache / rate limits / short-lived coordination — **not** a source of truth.

### 4.4 NATS (`nats`)

| | |
| --- | --- |
| **Image** | `nats:2.10-alpine` |
| **Client (host)** | `nats://127.0.0.1:4222` |
| **Client (Compose)** | `nats://nats:4222` |
| **Monitor** | [http://localhost:8222](http://localhost:8222) (`-js -m 8222`) |

JetStream enabled for future job/event streams. Official image is minimal (no in-container shell healthcheck).

### 4.5 Temporal (`temporal`)

| | |
| --- | --- |
| **Image** | `temporalio/auto-setup:1.25.2` |
| **gRPC (host)** | `127.0.0.1:7233` |
| **gRPC (Compose)** | `temporal:7233` |
| **Dynamic config** | `docker/compose/temporal/dynamicconfig/development-sql.yaml` |

Bootstraps schema against Postgres on first start. Workers/workflows are not registered yet (foundation).

### 4.6 Temporal UI (`temporal-ui`)

| | |
| --- | --- |
| **Image** | `temporalio/ui:2.31.2` |
| **URL** | [http://localhost:8088](http://localhost:8088) |
| **Backend** | `TEMPORAL_ADDRESS=temporal:7233` |

Mapped to **8088** on the host so it does not collide with the Rust API on **8080**.

### 4.7 Rust API (`api`)

| | |
| --- | --- |
| **Dockerfile** | `docker/Dockerfile.api.dev` |
| **Binary** | `proven-api` via `cargo watch -x run -p proven-api` |
| **URL** | [http://localhost:8080/healthz](http://localhost:8080/healthz) |
| **Also** | `/readyz`, `/api/v1/health` |
| **Bind mounts** | repo root → `/app` |
| **Caches** | Cargo registry + `/app/target` named volumes |

Edit `apps/api` or `crates/**` on the host; the container rebuilds/restarts the process.

### 4.8 Go worker (`worker`)

| | |
| --- | --- |
| **Dockerfile** | `docker/Dockerfile.workers.dev` |
| **Default binary** | `notify-worker` (Air hot reload) |
| **URL** | [http://localhost:8091/healthz](http://localhost:8091/healthz) |
| **Bind mounts** | `go/` → `/src` |

Foundation mode: HTTP health only (no Temporal activities yet). To run a different worker image:

```bash
docker compose -f docker/compose/docker-compose.yml --project-directory . \
  build --build-arg WORKER=media-worker worker
```

(Adjust published port / env to match that binary’s default port.)

### 4.9 Next.js (`web`)

| | |
| --- | --- |
| **Dockerfile** | `docker/Dockerfile.web.dev` |
| **URL** | [http://localhost:3000](http://localhost:3000) |
| **Bind mounts** | `apps/web`, `packages` |
| **Env** | `NEXT_PUBLIC_PROVEN_API_URL=http://localhost:8080` (browser) |
| | `PROVEN_API_URL=http://api:8080` (server / RSC inside Compose) |
| **Polling** | `WATCHPACK_POLLING` / `CHOKIDAR_USEPOLLING` for reliable HMR on Docker Desktop |

---

## 5. Hot reload notes

| Layer | Mechanism | Watches |
| --- | --- | --- |
| API | `cargo-watch` | `apps/api`, `crates` |
| Worker | [Air](https://github.com/air-verse/air) | `go/**/*.go` |
| Web | Next.js Fast Refresh | `apps/web`, `packages` |
| Infra | none | image upgrades only |

Infrastructure services do not hot-reload; recreate the container after compose/image changes.

---

## 6. Scripts

| Script | Action |
| --- | --- |
| [`scripts/dev/up.sh`](../../scripts/dev/up.sh) | Start full stack (`--deps-only`, `--build`, `--foreground`) |
| [`scripts/dev/down.sh`](../../scripts/dev/down.sh) | Stop stack (`--volumes` wipes DB/Redis data) |
| [`scripts/dev/logs.sh`](../../scripts/dev/logs.sh) | Follow logs (`logs.sh api worker`) |
| [`scripts/dev/ps.sh`](../../scripts/dev/ps.sh) | `compose ps` |

Compose always uses `--project-directory` = repo root so bind-mount paths resolve correctly.

---

## 7. Dependencies-only mode

Run infra while developing apps on the host:

```bash
./scripts/dev/up.sh --deps-only
make docker-deps
```

File: `docker/compose/docker-compose.deps.yml`  
Then: `make dev-api`, `make dev-web`, `make dev-worker-notify`.

---

## 8. Production-ish Dockerfiles

| File | Role |
| --- | --- |
| `docker/Dockerfile.api` | Multi-stage release `proven-api` |
| `docker/Dockerfile.workers` | Static Go worker binary (`WORKER` build-arg) |
| `docker/Dockerfile.web.dev` | Dev/HMR only (no production web image yet) |

Do not use `.dev` Dockerfiles for production deploys.

---

## 9. Troubleshooting

| Symptom | Likely fix |
| --- | --- |
| API healthcheck failing for minutes | First `cargo` build is slow; check `./scripts/dev/logs.sh api` |
| Web shows API unreachable | Ensure `api` is healthy; RSC uses `PROVEN_API_URL=http://api:8080` |
| Temporal UI empty/error | Wait for `temporal` auto-setup; confirm Postgres is healthy |
| Port already allocated | Stop host processes on 3000/8080/5432/… or change published ports |
| Stale DB state | `./scripts/dev/down.sh --volumes` (destructive) then `up.sh` |

---

## 10. Hard rules (unchanged)

- Go workers remain **I/O-only**; domain authority stays in Rust modules.  
- No secrets in git — use `.env` (gitignored).  
- AuthZ / compliance logic is **not** implemented in this stack yet.
