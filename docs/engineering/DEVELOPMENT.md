# Proven — Development Guide

| Field | Value |
| --- | --- |
| **Document type** | Local Development Guide |
| **Status** | Active (foundation scaffolding) |
| **Last updated** | 2026-08-03 |
| **Companion** | [GitHub Repository Design](./GITHUB_REPOSITORY.md), [Contributing](../../CONTRIBUTING.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

How to develop Proven locally. Foundation binaries (API, web, workers) start with health endpoints. Domain modules are not implemented yet.

---

## 2. Prerequisites

| Tool | Notes |
| --- | --- |
| **Git** | Current |
| **Node.js** ≥ 20.19 + **pnpm** 9.15 | Pinned in root `package.json` / `packageManager` |
| **Rust** | Via `rust-toolchain.toml` (1.86) |
| **Go** | 1.22+ (`go/go.mod`) |
| **Docker** + Compose | Optional — `make deps-up` |
| **Make** | Root task runner |

Never commit real secrets. Copy `.env.example` → `.env`.

---

## 3. Repository map

| Path | What you work on |
| --- | --- |
| `apps/web` | Next.js PWA / admin shell |
| `apps/api` + `crates/` | Rust modular monolith host |
| `go/` | I/O workers |
| `packages/` | Shared TS (`ui`, `api-client`, `pwa-sync`) |
| `contracts/` | OpenAPI, events, Temporal contracts |
| `db/` | Migrations / seeds |
| `docker/compose` | Postgres, Redis, NATS, Temporal |
| `docs/` | Architecture & product docs |

---

## 4. Bootstrap

```bash
cp .env.example .env
./scripts/dev/bootstrap.sh
# or
make bootstrap
```

| Task | Intent |
| --- | --- |
| `make bootstrap` | pnpm install + copy `.env` |
| `make deps-up` | Compose core dependencies |
| `make build` | Build API, workers, web |
| `make dev-api` | Axum on `:8080` |
| `make dev-web` | Next.js on `:3000` |
| `make dev-worker-notify` | notify-worker on `:8091` |
| `make check` | fmt/clippy/vet + typecheck |

---

## 5. Running components

### 5.1 Dependencies (optional)

```bash
make deps-up
# docker compose -f docker/compose/docker-compose.yml up -d
```

Services: PostgreSQL, Redis, NATS, Temporal. Foundation apps do not require them yet.

### 5.2 API (Rust)

```bash
make dev-api
# cargo run -p proven-api
curl http://127.0.0.1:8080/healthz
curl http://127.0.0.1:8080/api/v1/health
```

### 5.3 Web (Next.js)

```bash
make dev-web
# open http://127.0.0.1:3000
```

Talks to the API via `NEXT_PUBLIC_PROVEN_API_URL` and `@proven/api-client`.

### 5.4 Workers (Go)

```bash
cd go
go run ./cmd/notify-worker       # :8091
go run ./cmd/temporal-io-worker  # :8092
go run ./cmd/media-worker        # :8093
go run ./cmd/analytics-worker    # :8094
```

Health: `GET /healthz`. Foundation mode only — no Temporal/NATS jobs yet.

---

## 6. Contracts & codegen

- Edit OpenAPI / events under `contracts/`.
- Codegen scripts land under `scripts/codegen/` in later milestones.

---

## 7. Database

- Migrations in `db/migrations` (empty at foundation).
- `scripts/db/migrate.sh` is a no-op placeholder.

---

## 8. Dev containers

Open the repo in VS Code / Cursor with `.devcontainer/devcontainer.json` for a preconfigured Node + Rust + Go environment.

---

## 9. Hard rules (local)

- Domain authority stays in Rust modules (not Go, not the browser).
- AuthZ on the server; never trust client role claims.
- No secrets in git.
- See [AGENTS.md](../../AGENTS.md).
