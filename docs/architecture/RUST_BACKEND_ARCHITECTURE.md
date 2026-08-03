# Proven — Rust Backend Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Rust Backend Architecture |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Principal Rust Architecture |
| **Audience** | Backend Engineering, Platform, Security |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [System Architecture](./SYSTEM_ARCHITECTURE.md), [Repository Plan](./REPOSITORY_PLAN.md), [PostgreSQL Architecture](./POSTGRESQL_ARCHITECTURE.md), [Domain Model](./DOMAIN_MODEL.md), [Core Domain](./CORE_DOMAIN.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document defines the **Rust backend architecture** for Proven’s modular monolith API.

Stack: **Axum**, **Tower**, **Tokio**, **SQLx**, **Serde** (plus supporting crates for tracing, config, NATS, Temporal client, OpenAPI).

**Architecture only — no application code.**

---

## 2. Goals & Constraints

1. Host all domain modules in one deployable (`proven-api`) initially.  
2. Preserve **bounded-context independence** (public interfaces, events, Temporal—never cross-schema internals).  
3. Business rules live in **domain/application layers**, not handlers or workers.  
4. Strong typing at boundaries; fail closed on authz.  
5. Transaction + outbox for reliable domain events.  
6. Testable without network where practical (trait seams).  
7. Align with Postgres RLS (`app.tenant_id`), Soft deletes, audit append.  

---

## 3. Crate Layout

### 3.1 Workspace Members (Conceptual)

```text
Cargo workspace
├── apps/api                          # binary: proven-api (thin main)
├── crates/
│   ├── proven-shared                 # IDs, time, error envelope, correlation
│   ├── proven-platform               # host wiring, middleware, app state
│   ├── proven-contracts              # optional: OpenAPI/event type mirrors
│   ├── proven-test-support           # fixtures, test DB helpers
│   └── modules/
│       ├── proven-core
│       ├── proven-admin
│       ├── proven-projects
│       ├── proven-people
│       ├── proven-safety
│       ├── proven-equipment
│       ├── proven-documents
│       ├── proven-training
│       ├── proven-cor
│       ├── proven-signatures
│       ├── proven-notifications
│       ├── proven-analytics
│       └── proven-workflows
```

Naming may use `modules-*` aliases; one crate per bounded context.

### 3.2 Dependency Direction

```text
apps/api
  → proven-platform
      → each proven-<module> (composition root only)
proven-<module>
  → proven-shared
  → other modules ONLY via their public application traits (re-exported interfaces)
proven-platform
  → infrastructure adapters shared (db pool, nats, temporal, redis, r2)
```

**Forbidden:** `proven-safety` importing `proven-training` infrastructure/SQL; modules importing each other’s `domain` internals.

### 3.3 What Each Crate Owns

| Crate | Owns |
| --- | --- |
| `proven-shared` | `TenantId`, `ProjectId`, …; `AppError` skeleton; correlation types; paging |
| `proven-platform` | Router composition, middleware stack, `AppState`, config load, outbox publisher task, health endpoints |
| `proven-<module>` | Domain model, use cases, repos traits+SQLx impl, HTTP route registration, public module API traits |
| `proven-contracts` | Versioned DTO/OpenAPI shared with codegen consumers (optional) |
| `proven-test-support` | Testcontainers helpers, assert helpers |

---

## 4. Folder Structure

### 4.1 Binary Host

```text
apps/api/
  src/main.rs          # load config, build state, serve
  Cargo.toml
```

### 4.2 Platform Crate

```text
crates/proven-platform/
  src/
    lib.rs
    config/
    state/             # AppState, ModuleHandles
    http/
      router.rs        # nest module routers, /health, /ready
      middleware/      # authn, request-id, tenant, tracing, timeout
    infra/
      db.rs            # pool, RLS session setup
      nats.rs
      temporal.rs
      redis.rs
      r2.rs
      outbox_worker.rs
    openapi.rs         # merge module OpenAPI docs
```

### 4.3 Module Crate (Canonical)

```text
crates/modules/proven-safety/   # example
  src/
    lib.rs
    domain/            # aggregates, VOs, domain events, domain services
    application/       # commands, queries, public ports (traits)
      ports.rs         # outbound: repos, clock, event sink, other-module APIs
      commands/
      queries/
      public.rs        # Public SafetyApi trait consumed by other modules
    infrastructure/
      sqlx/            # repository implementations, mappers
      nats/            # optional module-specific consumers
    http/
      handlers/
      dto/
      routes.rs
      openapi.rs
    error.rs           # module error → platform error mapping
  tests/               # integration tests
```

Same shape for every module. Layers explained in §5.

---

## 5. Layering Explained

### 5.1 HTTP / Handler Layer (Axum)

**Responsibility:** Translate HTTP ↔ application use cases.

- Parse path/query/body into **DTOs**  
- Extract `RequestContext` (tenant, principal, correlation)  
- Call application command/query services  
- Map results to HTTP status + response DTOs  
- Map errors to stable problem responses  

**Must not:** contain business invariants, SQL, or direct NATS publish.

### 5.2 Application Layer (Use Cases / Services)

**Responsibility:** Orchestrate a single use case.

- Authorize via Core `AuthzApi` (or pre-checked gate + re-check)  
- Load aggregates via repository ports  
- Invoke domain methods  
- Persist via unit of work / transaction  
- Register domain events to outbox  
- Call other modules’ **public traits** when needed  
- Start/signal Temporal workflows when process requires durability  

This is the primary **service** layer (application services). Domain services stay in `domain/` for pure multi-aggregate rules without I/O.

### 5.3 Domain Layer

**Responsibility:** Enterprise business rules.

- Aggregates, entities, value objects  
- Invariants and state transitions  
- Domain events (in-memory facts)  
- Domain services (pure)  

**Must not:** depend on Axum, SQLx, NATS, Serde DTOs for HTTP (Serde may appear on shared event envelopes carefully—prefer domain types free of transport).

### 5.4 Infrastructure Layer

**Responsibility:** Adapters for Postgres, buses, object storage, Temporal, Redis.

- SQLx repositories  
- Outbox writer  
- NATS publisher/consumer adapters  
- Temporal client wrappers  
- R2 presign adapters  

Implements ports defined by application layer (**dependency inversion**).

### 5.5 Public Module API Layer

**Responsibility:** In-process façade for other modules and Temporal activities.

Example capabilities: `ProjectsQueryApi`, `TrainingCompetencyApi`, `SignaturePackageApi`, `CoreAuthzApi`.

Implemented by application services; consumed as `Arc<dyn Trait>` in `AppState`.

---

## 6. Dependency Injection

### 6.1 Approach

**Manual composition root** in `proven-platform` (no heavy DI framework required).

At startup:

1. Load `Config`  
2. Build shared infra: `PgPool`, NATS connection, Temporal client, Redis, R2  
3. Construct each module’s repositories + services  
4. Register `Arc<dyn ModulePublicApi>` handles on `AppState`  
5. Build Axum `Router` with state  

### 6.2 AppState Shape (Logical)

- Shared: pool, config, clock, outbox, nats, temporal, metrics  
- Module handles: `core_api`, `projects_api`, `safety_api`, …  
- Policy: handlers receive `State<AppState>` and extract only what they need  

### 6.3 Tower Integration

Tower middleware layers wrap the router for:

- Timeouts  
- Concurrency limits  
- Trace/metrics  
- AuthN / request context  
- CORS / compression (as needed)  

Services remain plain async Rust traits—not Tower services—for domain clarity.

### 6.4 Testing Seams

Ports are traits:

- `SafetyActivityRepository`  
- `UnitOfWork` / `Transaction`  
- `EventSink` (outbox)  
- `Clock`  
- `TrainingCompetencyApi`  

Tests inject fakes; integration tests inject SQLx against ephemeral Postgres.

---

## 7. Repositories

### 7.1 Role

Persist and load aggregates / read models for **one schema**.

### 7.2 Design Rules

1. One repository family per aggregate (or cohesive cluster).  
2. Methods speak domain language (`save`, `get`, `list_open_by_project`).  
3. SQL lives only in infrastructure.  
4. Respect soft deletes and tenant_id.  
5. Set Postgres session GUC `app.tenant_id` at transaction start (platform helper).  
6. Optimistic concurrency via `row_version`.  
7. No cross-schema joins.  

### 7.3 Mapping

- Row structs (SQLx `FromRow`) ≠ domain aggregates  
- Explicit mappers domain ↔ row  
- JSONB columns map to typed value objects where possible  

### 7.4 Read Models

Separate query repositories or methods for list/detail DTOs when aggregate load is too heavy (CQRS-lite inside module).

---

## 8. Services

| Kind | Location | I/O | Examples |
| --- | --- | --- | --- |
| **Application service** | `application/` | Yes (ports) | `SubmitSafetyActivity`, `PublishDocumentVersion` |
| **Domain service** | `domain/` | No | Eligibility composition helpers local to module; risk rating pure calc |
| **Infrastructure service** | `infrastructure/` | Yes | `OutboxPublisher`, `NatsEventBus`, `TemporalWorkflowStarter` |

Application services are the default “service” collaborators for handlers.

**Lifecycle of a write use case:**

```text
Handler
  → ApplicationService.execute(cmd, ctx)
      → AuthzApi.authorize
      → begin transaction + set RLS GUC
      → repo.load
      → aggregate.domain_method
      → repo.save
      → outbox.add(domain_events)
      → optional temporal.start
      → AuditApi.append
      → commit
  → map to response DTO
```

---

## 9. Handlers

### 9.1 Conventions

- One handler function per route endpoint (or small group)  
- Extractors: `State`, `Extension<RequestContext>`, `Path`, `Query`, `Json`, `TypedHeader`  
- Return `Result<impl IntoResponse, AppError>`  
- Idempotency: read `Idempotency-Key` header for offline/write routes → platform idempotency store  

### 9.2 Route Registration

Each module exports `router() -> Router<AppState>` nested under versioned prefix by platform.

Admin/BFF aggregation handlers live in `proven-admin` or platform only when composing multiple modules for Admin Dashboard.

---

## 10. DTOs

### 10.1 Categories

| DTO | Direction | Notes |
| --- | --- | --- |
| **Request DTO** | HTTP → app | Serde deserialize; validation |
| **Response DTO** | app → HTTP | Serde serialize; stable public shape |
| **Command/Query models** | app internal | May be DTO-mapped 1:1 or richer |
| **Event payload DTO** | outbox/NATS | Versioned, additive |
| **Public API DTO** | module↔module | Decision-oriented, not ORM rows |

### 10.2 Rules

- Never expose row structs directly  
- Never expose password hashes, magic link secrets, raw provider payloads  
- Use newtype IDs from `proven-shared` in public JSON as UUID strings  
- Pagination envelope standard (`items`, `next_cursor`)  

---

## 11. Validation

### 11.1 Layers

1. **Syntactic (HTTP):** required fields, UUID parse, string lengths — request DTO validation (e.g. `validator` crate or manual).  
2. **Application:** cross-field checks, authz, existence via ports.  
3. **Domain:** invariants on aggregate transitions (authoritative).  

### 11.2 Rules

- UI/Zod validation is non-authoritative.  
- Domain errors distinct from validation errors (409/422/400 mapping).  
- JSONB payloads validated against schema version for activity responses / builder drafts.

---

## 12. Authentication

### 12.1 Responsibilities (Core module + platform middleware)

| Concern | Owner |
| --- | --- |
| Session/JWT/OIDC verification | Core identity + platform AuthN middleware |
| Principal resolution | Core |
| Guest/magic-link token auth | Signatures/Documents routes with specialized extractors |
| Service/API keys | Admin/Core identity integration |

### 12.2 Middleware Flow

```text
Request
  → RequestId/Correlation
  → AuthN (establish Principal or GuestToken)
  → Load tenant membership / bind tenant_id
  → Attach RequestContext extension
  → Handler
```

Unauthenticated public routes: health, OIDC callbacks, magic-link redeem (tightly scoped).

### 12.3 RequestContext (Logical Contents)

- `tenant_id`, `principal_id`, `user_id?`, `person_id?`  
- `session_id?`, `assurance_level?`  
- `correlation_id`, `causation_id`  
- `roles/grants cache handle` (optional)  
- `ip/user_agent` (audit meta, policy-limited)  

---

## 13. Authorization

### 13.1 Model

All authorization decisions via **Core AuthzApi** (`Authorize(principal, permission_code, scope)`).

Handlers may do coarse gate; application services re-check for sensitive commands.

### 13.2 Tower/AuthZ Gate

Optional middleware for route-level permission declarations; still not a substitute for resource-scoped checks (project membership, document ACL).

### 13.3 Patterns

- Load resource → derive scope (`ProjectScope(project_id)`) → authorize  
- List endpoints filter by authorized scopes (query-level), never rely on client filters alone  
- Module entitlement: `LicenseApi.is_module_enabled` before feature routes  

---

## 14. Configuration

### 14.1 Sources

- Environment variables / secret store  
- Optional layered file config for local  
- 12-factor style  

### 14.2 Logical Config Sections

- `server` (bind, timeouts)  
- `database` (URL, pool size)  
- `redis`  
- `nats`  
- `temporal` (host, namespace)  
- `r2` (bucket, credentials)  
- `auth` (OIDC, cookie, JWT)  
- `observability` (OTLP, log level)  
- `feature` defaults  

### 14.3 Rules

- Typed config struct validated at boot (fail fast)  
- Secrets never logged  
- Per-environment via deploy (Fly/Vercel not for API secrets in git)  

---

## 15. Logging & Observability

### 15.1 Logging

- Structured logging (`tracing`) with JSON in prod  
- Fields: `correlation_id`, `tenant_id`, `principal_id`, `module`, `error_code`  
- PII/PHI redaction policies  
- No magic link tokens, passwords, signature strokes  

### 15.2 Metrics & Traces

- RED metrics per route and use case  
- DB pool metrics  
- Outbox lag, NATS publish failures, Temporal schedule failures  
- Distributed traces: HTTP → service → SQL → external  

Health: `/health` liveness; `/ready` checks pool + NATS + Temporal reachability.

---

## 16. Error Handling

### 16.1 Error Taxonomy

| Class | HTTP | Examples |
| --- | --- | --- |
| Validation | 400/422 | Bad DTO |
| Unauthenticated | 401 | Missing session |
| Forbidden | 403 | AuthZ deny |
| Not found | 404 | Missing aggregate (no leak across tenant) |
| Conflict | 409 | Version conflict, duplicate mutation |
| Precondition | 412/428 | Idempotency / version pin |
| Domain reject | 422 | Invariant violation |
| Downstream | 502/503 | Temporal/NATS soft deps when required |
| Internal | 500 | Unexpected |

### 16.2 Design

- `AppError` in platform with stable `code` string  
- Module errors convert via `From` / mapper  
- Problem+JSON or consistent JSON error envelope  
- Domain violations never become 500  
- AuthZ deny audited when policy requires  

---

## 17. Transactions

### 17.1 Unit of Work

- One Postgres transaction per write use case by default  
- Begin → set `app.tenant_id` → work → outbox insert → audit insert → commit  
- Do **not** hold transactions across external HTTP/Temporal awaits  

### 17.2 Cross-Module Consistency

- No distributed DB transactions across schemas  
- Use outbox events + Temporal for multi-module processes  
- In-process cross-module calls within same request either:  
  - participate carefully in same transaction **only if** same pool connection passed (advanced; prefer avoid), or  
  - commit module A then call module B with compensating workflow on failure  

**Preferred:** single module write + events; orchestration in Temporal.

### 17.3 Isolation

- Default `READ COMMITTED`  
- Explicit locking for contested aggregates when needed  

---

## 18. Testing

### 18.1 Pyramid

| Layer | Scope | Tools (conceptual) |
| --- | --- | --- |
| Domain unit | Aggregates/invariants | Rust tests, no I/O |
| Application unit | Use cases with fake ports | Trait mocks/fakes |
| Repository integration | SQLx against Postgres | Testcontainers / ephemeral DB |
| HTTP contract | Axum router tests | Request/response + auth context |
| Module contract | Public API traits | Cross-module fake or integration |
| E2E | Sparse critical journeys | External suite |

### 18.2 Rules

- Every core aggregate gets invariant tests  
- Migration applied in integration setup  
- RLS tests: wrong tenant cannot read  
- Idempotency tests for offline writes  
- No reliance on wall-clock without `Clock` port  

---

## 19. Domain Events

### 19.1 Flow

```text
Aggregate raises DomainEvent (in memory)
  → Application collects events
  → Outbox rows in same transaction
  → Commit
  → Outbox publisher (platform task) publishes to NATS
  → Consumers (notifications, analytics, cor, projections)
```

### 19.2 Envelope

Shared fields: `event_id`, `event_type`, `event_version`, `occurred_at`, `tenant_id`, `actor`, `correlation_id`, `causation_id`, `resource`, `payload`.

### 19.3 Rules

- Past-tense names  
- Additive versioning  
- No foreign aggregate internals  
- At-least-once delivery; consumers idempotent  

---

## 20. NATS Integration

### 20.1 Publisher Path

- Platform outbox worker reads unpublished rows  
- Publishes to subject namespace `proven.<module>.v<major>.<event>`  
- Marks published / retries with backoff  
- Metrics on lag  

### 20.2 Consumer Path (in Rust)

- Some projections/consumers may run in-process (careful)  
- Prefer Go workers for analytics ETL & notification send  
- Rust consumers for module projections that must stay near OLTP (e.g., readiness recompute triggers) use durable queue groups  

### 20.3 Rules

- Handlers do not publish directly to NATS (outbox only)  
- Serialization via Serde JSON  
- Poison message policy → DLQ subject + alert  

---

## 21. Temporal Client

### 21.1 Placement

- Client wrapper in `proven-platform` / `proven-workflows`  
- Application services start/signal workflows through a port: `WorkflowPort`  
- Activities implemented as workers calling **public module APIs** (same process or activity worker binary later)  

### 21.2 Rules

- Activities are idempotent and thin  
- Activities never use private repos of another module  
- Do not await long workflows inside HTTP request (start + return run id)  
- Workflow names/versioning owned by workflows module catalog  

### 21.3 Failure

- Start failures → 503 or deferred outbox “workflow intent” if required for durability  

---

## 22. API Versioning

### 22.1 Strategy

- URL prefix: `/api/v1/...`  
- Modules nested: `/api/v1/safety/...`, `/api/v1/projects/...`  
- Additive changes within `v1`  
- Breaking changes require `/api/v2` with coexistence window  

### 22.2 Header Alternatives

- Optional `Accept-Version` later; URL remains source of truth initially  

### 22.3 Deprecation

- OpenAPI mark deprecated  
- Emit warning headers  
- Remove only after consumer migration + ADR  

---

## 23. OpenAPI

### 23.1 Approach

- Generate/annotate per-module OpenAPI fragments  
- Platform merges into single `openapi.json` for `v1`  
- DTOs documented with examples and error codes  
- Security schemes: bearer/cookie/API key  

### 23.2 Publishing

- Served at `/api/v1/openapi.json` (authz optional for public subset)  
- Contracts also copied/synced to `contracts/openapi/` in monorepo for codegen  

### 23.3 Rules

- OpenAPI is the HTTP contract source for web clients  
- In-process module APIs are separate from OpenAPI (traits)  
- CI fails on unintentional breaking OpenAPI diffs  

---

## 24. Axum + Tower + Tokio + SQLx + Serde Roles

| Technology | Role |
| --- | --- |
| **Tokio** | Async runtime; tasks for outbox publisher, background cleans |
| **Axum** | HTTP routing, extractors, responses |
| **Tower** | Middleware layered services (timeout, trace, auth) |
| **SQLx** | Async Postgres access, migrations runner (or companion), compile-time or runtime checked queries per team standard |
| **Serde** | DTO and event JSON serialization |

SQLx note: prefer runtime-checked queries in early modular monolith for migration velocity, or checked macros in CI with `DATABASE_URL`—document team choice in ADR.

---

## 25. End-to-End Request Path (Summary)

```text
Client
  → Cloudflare/TLS
  → Axum (Tower stack)
  → AuthN middleware → RequestContext
  → Module handler (DTO validate)
  → Application service
      → Core AuthZ
      → SQLx transaction + RLS GUC
      → Domain aggregate
      → Outbox + Audit
      → Temporal start (optional)
  → Commit
  → Response DTO
Async: Outbox → NATS → Notifications/Analytics/COR/…
```

---

## 26. Security Checklist (Backend)

- [ ] Every write authorized  
- [ ] Tenant GUC set on all DB txs  
- [ ] No cross-module SQL  
- [ ] Secrets from env/secret store  
- [ ] Audit on compliance-significant commands  
- [ ] Idempotency for offline writes  
- [ ] Error responses do not leak existence across tenants  

---

## 27. Success Criteria

The Rust backend architecture succeeds when:

1. New domain features land in one module crate without editing others’ internals.  
2. Handlers stay thin; domain tests cover invariants without HTTP.  
3. Events are reliably published via outbox.  
4. Temporal activities call only public APIs.  
5. OpenAPI + `/api/v1` evolve safely.  
6. AuthN/AuthZ and RLS form layered defense.  
7. The binary remains a modular monolith ready for future extraction at trait/event seams.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Principal Rust Architecture | Complete Rust/Axum backend architecture (no code) |

---

*End of Rust Backend Architecture*
