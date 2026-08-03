# Proven — Development Guide

| Field | Value |
| --- | --- |
| **Document type** | Local Development Guide |
| **Status** | Active (foundation scaffolding) |
| **Last updated** | 2026-08-03 |
| **Companion** | [**Developer Handbook (canonical)**](../development/README.md), [GitHub Repository Design](./GITHUB_REPOSITORY.md), [Contributing](../../CONTRIBUTING.md), [AGENTS.md](../../AGENTS.md) |

---

## Canonical onboarding

**Use the [Developer Handbook](../development/README.md)** for Getting Started, Local Development, Commit Conventions, Architecture Gates, Testing, and Editor Setup.

Quick path:

```bash
just setup          # or: ./scripts/dev/setup.sh
just api            # Rust API :8080
just web            # Next.js :3000
just worker notify  # Go notify-worker
```

This page remains a short pointer plus environment/Docker companions. Prefer `just` recipes (`Makefile` wraps them).

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
| **Docker** + Compose | `just up` / `just deps` — see [Docker Local Development](./DOCKER_LOCAL_DEVELOPMENT.md) |
| **just** | Primary task runner (`brew install just`); Make wrappers optional |
| **lefthook** | Git hooks (`just hooks`) |

Never commit real secrets. Copy `.env.example` → `.env`. See [Environment Configuration](./ENVIRONMENT_CONFIGURATION.md).

---

## 3. Repository map

| Path | What you work on |
| --- | --- |
| `apps/web` | Next.js PWA / admin shell |
| `apps/api` + `crates/` | Rust modular monolith host |
| `go/` | I/O workers |
| `packages/` | Shared TS (`ui`, `api-client`, `pwa-sync`) |
| `contracts/` | OpenAPI, events, Temporal contracts |
| `db/` | Migrations / seeds / fixtures |
| `docker/compose` | Postgres, Redis, NATS, Temporal |
| `docs/development/` | Developer handbook |
| `docs/architecture/` | Architecture deep-dives |

---

## 4. Bootstrap

```bash
just setup
# or deps-only (devcontainer):
just setup --deps-only
```

| Task | Intent |
| --- | --- |
| `just setup` | One-command onboarding |
| `just up` | Full Docker Compose stack |
| `just deps` | Infra only |
| `just down` | Stop Compose |
| `just build` | Build API, workers, web |
| `just api` / `just web` / `just worker notify` | Host processes |
| `just check` / `just ci` / `just arch` | Quality gates |
| `just docs` | Export OpenAPI + handbook pointer |

Make targets (`make bootstrap`, `make docker-up`, …) call the same `just` recipes when `just` is installed.

---

## 5. Running components

### 5.1 Docker (recommended)

```bash
just up
# docs: Docker Local Development
```

Full catalog: [DOCKER_LOCAL_DEVELOPMENT.md](./DOCKER_LOCAL_DEVELOPMENT.md).

### 5.2 Dependencies only (host apps)

```bash
just deps
just api   # terminal 1
just web   # terminal 2
just worker notify
```

### 5.3 API (Rust)

```bash
just api
curl -i http://127.0.0.1:8080/health
open http://127.0.0.1:8080/docs     # Swagger
open http://127.0.0.1:8080/redoc    # Redoc
```

See [API Documentation](../development/API_DOCUMENTATION.md).

### 5.4 Web / Workers

See [Local Development](../development/LOCAL_DEVELOPMENT.md) and [go/README.md](../../go/README.md).

---

## 6. Contracts & codegen

```bash
just docs
# → contracts/openapi/openapi.json
```

---

## 7. Database

```bash
just db-migrate
just db-seed local
just db-reset
just db-migrate-create my_change
```

Fixtures (non-executable): `db/seeds/fixtures/`. See [Seeds & Fixtures](../development/SEEDS_AND_FIXTURES.md).

---

## 8. Dev containers

Open with `.devcontainer/devcontainer.json` — `postCreateCommand` runs `scripts/dev/setup.sh --deps-only`. Editor details: [Editor Setup](../development/EDITOR_SETUP.md).

---

## 9. Hard rules (local)

- Domain authority stays in Rust modules (not Go, not the browser).
- AuthZ on the server; never trust client role claims.
- No secrets in git.
- See [AGENTS.md](../../AGENTS.md).

---

## 10. Metrics

[Engineering Metrics](../development/ENGINEERING_METRICS.md).
