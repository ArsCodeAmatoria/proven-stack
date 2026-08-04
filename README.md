<p align="center">
  <img src="assets/brand/proven-mark.png" alt="Proven" width="112" height="112" />
</p>

<h1 align="center">Proven</h1>

<p align="center">
  <img src="https://img.shields.io/badge/Construction%20Compliance%20Operating%20System-C9A227?style=for-the-badge&labelColor=3F3A32&color=C9A227" alt="Construction Compliance Operating System" /><br />
  Defensible proof that people, equipment, and work are compliant—every day, on every site.
</p>

<p align="center">
  <img alt="Earthy · Mustard accent" src="https://img.shields.io/badge/brand-earthy%20%C2%B7%20mustard-C9A227?style=flat-square&labelColor=3F3A32" />
  <img alt="API v1" src="https://img.shields.io/badge/API-v1-8B7355?style=flat-square&labelColor=3F3A32" />
  <img alt="Modular monolith" src="https://img.shields.io/badge/architecture-modular%20monolith-6B5E4E?style=flat-square&labelColor=3F3A32" />
</p>

Proven is built for General Contractors, Prime Contractors, Subcontractors, Crane Companies, Concrete Forming Companies, Civil Contractors, and Industrial Contractors across
<img src="https://img.shields.io/badge/Canada%20·%20United%20States%20·%20Australia%20·%20New%20Zealand-C9A227?style=flat-square&labelColor=3F3A32&color=C9A227" alt="Canada, the United States, Australia, and New Zealand" />.

Mobile-first for workers. Desktop-first for supervisors, safety coordinators, project managers, and administrators.

> Logo mark adapted from Lucide [`fingerprint-pattern`](https://lucide.dev/icons/fingerprint-pattern) (ISC). White fingerprint on earthy `#3F3A32` (no ring). Highlight text: mustard `#C9A227`.

---

## What it is

Proven is not a forms app. It is one cohesive platform for:

- Projects & people
- Safety operations
- Equipment compliance
- Documents & digital signatures
- Training & competency
- COR audit readiness
- Workflows, notifications, analytics, and administration

---

## Repository status

Monorepo **platform foundation is in place**: Core (tenancy, AuthZ, audit, files), Companies, Users, and Projects modules; NATS event library; Temporal infrastructure (no workflows yet); REST API conventions (`/api/v1`). Safety / Equipment / Documents modules and full Better Auth wiring remain ahead.

```text
proven-stack/
├── apps/              # web (Next.js), api (Rust), workers (pointer to go/)
├── crates/            # proven-shared, proven-platform, proven-events, proven-temporal
│   └── modules/       # proven-core, proven-companies, proven-users, proven-projects
├── go/                # I/O worker binaries
├── packages/          # ui, api-client, pwa-sync, shared configs
├── docker/            # Dockerfiles + compose
├── contracts/ db/ deploy/ infra/ scripts/ tests/
├── docs/              # PRD, architecture, ADRs, developer handbook
├── .github/workflows/ # CI
├── .vscode/ .devcontainer/
├── Makefile / justfile
└── .env.example
```

Full layout: [GitHub Repository Design](./docs/engineering/GITHUB_REPOSITORY.md).

### Quick start

**Docker (recommended):**

```bash
cp .env.example .env
./scripts/dev/up.sh          # full stack with hot reload
# Web http://localhost:3000 · API :8080 · Temporal UI :8088
./scripts/dev/down.sh
```

See [Docker Local Development](./docs/engineering/DOCKER_LOCAL_DEVELOPMENT.md) for every service.

**On the host:**

```bash
./scripts/dev/bootstrap.sh   # or: make bootstrap / just setup
make docker-deps             # Postgres/Redis/NATS/Temporal/UI
make dev-api                 # http://127.0.0.1:8080/healthz
make dev-web                 # http://127.0.0.1:3000  (Node >= 20.19)
make dev-worker-notify       # http://127.0.0.1:8091/healthz
```

Requires for host mode: Rust (`rust-toolchain.toml`), Go 1.22+, Node.js ≥ 20.19 + pnpm 9.15, Docker (for deps or full stack).

---

## Intended stack

| Layer | Technology |
| --- | --- |
| Web / PWA | Next.js, TypeScript, Tailwind, shadcn/ui |
| API | Rust, Axum (modular monolith) |
| Workers | Go |
| Workflows | Temporal |
| Events | NATS |
| Data | PostgreSQL, Redis (cache only), Cloudflare R2, ClickHouse |
| Edge / deploy | Cloudflare, Vercel, Fly.io, Docker, GitHub Actions |
| IaC (future) | Terraform under `infra/terraform` |

Architecture style: **modular monolith**. Modules own their domains and integrate through public interfaces, events, and Temporal workflows—never by reaching into another module’s internals.

---

## Documentation

### Start here

| Doc | Description |
| --- | --- |
| [Implementation Roadmap](./docs/architecture/IMPLEMENTATION_ROADMAP.md) | **CTO roadmap:** milestones, MVP, path to enterprise |
| [AGENTS.md](./AGENTS.md) | Engineering principles and hard constraints |
| [Developer Handbook](./docs/development/README.md) | Canonical onboarding (`just setup`) |
| [REST API Conventions](./docs/development/REST_API_CONVENTIONS.md) | Envelopes, paging, AuthN/Z, rate limits, OpenAPI |
| [Contributing](./CONTRIBUTING.md) | Branching, PRs, review expectations |
| [Development Guide](./docs/engineering/DEVELOPMENT.md) | Pointer + Docker companions |
| [Docker Local Development](./docs/engineering/DOCKER_LOCAL_DEVELOPMENT.md) | Compose stack, ports, hot reload |
| [Environment Configuration](./docs/engineering/ENVIRONMENT_CONFIGURATION.md) | Typed config, secrets validation, envs |
| [Security Policy](./SECURITY.md) | Vulnerability reporting |
| [GitHub Repository Design](./docs/engineering/GITHUB_REPOSITORY.md) | Monorepo layout, CI, release, labels, CODEOWNERS |
| [CI & Branch Protection](./docs/engineering/CI_AND_BRANCH_PROTECTION.md) | PR Validation, artifacts, required checks |

### Product & UX

| Doc | Description |
| --- | --- |
| [PRD](./docs/PRD.md) | Product requirements |
| [UX Architecture](./docs/ux/UX_ARCHITECTURE.md) | Information architecture |
| [Design System](./docs/design/DESIGN_SYSTEM.md) | Visual and interaction system |

### Platform architecture

| Doc | Description |
| --- | --- |
| [System Architecture](./docs/architecture/SYSTEM_ARCHITECTURE.md) | Runtime & deployment |
| [Repository Plan](./docs/architecture/REPOSITORY_PLAN.md) | Structural engineering plan |
| [Core Domain](./docs/architecture/CORE_DOMAIN.md) | Tenancy, identity, AuthZ, files, audit |
| [REST API](./docs/architecture/REST_API.md) | `/api/v1` conventions |
| [Event Catalog](./docs/architecture/EVENT_CATALOG.md) | NATS domain events |
| [Temporal Workflows](./docs/architecture/TEMPORAL_WORKFLOWS.md) | Durable processes |
| [Security Architecture](./docs/architecture/SECURITY_ARCHITECTURE.md) | AuthN/Z, encryption, OWASP |
| [Security Review](./docs/architecture/SECURITY_ARCHITECTURE_REVIEW.md) | Architectural review + recommendations |
| [Authentication](./docs/architecture/AUTHENTICATION_ARCHITECTURE.md) | Better Auth, JWT, OAuth, MFA, guest, offline |
| [Authorization / RBAC](./docs/architecture/AUTHORIZATION_RBAC_ARCHITECTURE.md) | Roles, scopes, delegation, temporary grants |
| [Audit Logging](./docs/architecture/AUDIT_LOGGING_ARCHITECTURE.md) | Immutable audit: auth, signatures, approvals, exports |
| [Digital Signatures](./docs/architecture/DIGITAL_SIGNATURES_ARCHITECTURE.md) | Guest/magic/QR, chains, hashes, certificates, offline |
| [Notifications](./docs/architecture/NOTIFICATION_ARCHITECTURE.md) | Channels, quiet hours, digest, escalation, prefs |
| [PWA](./docs/architecture/PWA_ARCHITECTURE.md) | Install, offline, push, camera, QR, sync updates |
| [R2 Storage](./docs/architecture/R2_STORAGE_ARCHITECTURE.md) | Cloudflare R2: keys, lifecycle, retention, security |
| [AI Systems](./docs/architecture/AI_SYSTEMS_ARCHITECTURE.md) | RAG, assistants, pgvector, human review |
| [Integrations](./docs/architecture/INTEGRATION_FRAMEWORK_ARCHITECTURE.md) | Connectors, webhooks, REST, secrets, retry |
| [Testing Strategy](./docs/architecture/TESTING_STRATEGY.md) | Rust/Go/Web, Playwright, security, load, CI |
| [Deployment](./docs/architecture/DEPLOYMENT_ARCHITECTURE.md) | Dev→prod, Vercel, Fly, Cloudflare, rollback |
| [Observability](./docs/architecture/OBSERVABILITY_ARCHITECTURE.md) | OTel, Prometheus, Loki, Grafana, incidents |
| [Performance](./docs/architecture/PERFORMANCE_ARCHITECTURE.md) | Budgets: web, API, search, offline, scale |
| [Offline Sync](./docs/architecture/OFFLINE_SYNC_ARCHITECTURE.md) | PWA offline-first |
| [Search](./docs/architecture/SEARCH_ARCHITECTURE.md) | FTS → OpenSearch / pgvector |
| [Data Warehouse](./docs/architecture/DATA_WAREHOUSE_ARCHITECTURE.md) | ClickHouse analytics |
| [PostgreSQL](./docs/architecture/POSTGRESQL_ARCHITECTURE.md) | OLTP design |
| [Database Migrations](./docs/architecture/DATABASE_MIGRATION_STRATEGY.md) | Naming, expand/contract, seeds, prod |
| [Rust Backend](./docs/architecture/RUST_BACKEND_ARCHITECTURE.md) | Axum modular monolith |
| [Rust Crate Catalog](./docs/architecture/RUST_CRATE_CATALOG.md) | Every crate: API, deps, events, DB, tests |
| [Go Workers](./docs/architecture/GO_WORKERS_ARCHITECTURE.md) | I/O workers architecture |
| [Go Worker Catalog](./docs/architecture/GO_WORKER_CATALOG.md) | Every worker: Temporal, notify, PDF, OCR, retries |
| [Frontend](./docs/architecture/FRONTEND_ARCHITECTURE.md) | Next.js / PWA |
| [Frontend Folders](./docs/architecture/FRONTEND_FOLDER_STRUCTURE.md) | App Router tree; every folder documented |

### Domain architecture

| Doc | Module |
| --- | --- |
| [Domain Model](./docs/architecture/DOMAIN_MODEL.md) | Context map |
| [Projects](./docs/architecture/PROJECTS_DOMAIN.md) | Places |
| [People](./docs/architecture/PEOPLE_DOMAIN.md) | Workforce |
| [Safety](./docs/architecture/SAFETY_DOMAIN.md) | FLHA, CA, incidents |
| [Equipment](./docs/architecture/EQUIPMENT_DOMAIN.md) | Assets, readiness |
| [Documents](./docs/architecture/DOCUMENTS_DOMAIN.md) | Controlled docs / SWP |
| [Signatures](./docs/architecture/SIGNATURES_DOMAIN.md) | Proof of assent |
| [Training](./docs/architecture/TRAINING_DOMAIN.md) | Competency |
| [COR](./docs/architecture/COR_DOMAIN.md) | Audit readiness |
| [Notifications](./docs/architecture/NOTIFICATIONS_DOMAIN.md) | Channels |
| [Analytics](./docs/architecture/ANALYTICS_DOMAIN.md) | Insights product |
| [Administration](./docs/architecture/ADMINISTRATION_DOMAIN.md) | Admin facade |

---

## Core principles

- Domain-driven, API-first, security-first, offline-first
- Business rules live in domain modules—not in React or Go workers
- Redis is never permanent storage
- Never bypass Temporal for durable business workflows
- Never bypass audit logging
- Prefer long-term maintainability over short-term convenience

---

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md). Security reports: [SECURITY.md](./SECURITY.md).

---

## License

License TBD.
