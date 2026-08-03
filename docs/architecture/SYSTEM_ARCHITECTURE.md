# Proven — System Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | System Architecture |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Principal Architecture |
| **Audience** | Engineering, Security, SRE, Product |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [PRD](../PRD.md), [Domain Model](./DOMAIN_MODEL.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document defines the **complete technical architecture** for Proven, a Construction Compliance Operating System built as a **modular monolith** with clear runtime boundaries.

It covers system context, containers, components, module relationships, workflow, notifications, storage, authn/authz, offline sync, caching, deployment, security, disaster recovery, and scalability.

**No application code is included.** Domain boundaries are defined in the [Domain Model](./DOMAIN_MODEL.md); this document defines how those domains run on the chosen stack.

---

## 2. Architecture Principles

1. **Modular monolith first** — one Rust/Axum deployable hosts domain modules; extract services only when seams and scale demand it.
2. **API first** — Next.js and workers consume versioned HTTP/JSON (and signed upload) contracts; no embedded business rules in clients.
3. **Domain ownership** — each module owns its schema, commands, and invariants.
4. **Integration triad** — modules collaborate via **public interfaces**, **NATS events**, and **Temporal workflows** only.
5. **Durable process** — multi-step compliance processes run on Temporal; never client-only orchestration.
6. **Workers are execution, not judgment** — Go workers deliver notifications, transform/load analytics, and run I/O-heavy jobs; they do not own business rules.
7. **PostgreSQL is operational system of record** — Redis is cache/ephemeral only; R2 stores binaries; ClickHouse stores analytical projections.
8. **Security and audit by default** — every compliance-significant action is authenticated, authorized, and audited.
9. **Offline-first field paths** — worker PWA syncs through idempotent module commands.
10. **Region-aware cloud topology** — Cloudflare edge + Vercel web + Fly.io API/workers, with DR and backup discipline.

---

## 3. Technology Roles

| Technology | Architectural role |
| --- | --- |
| **Next.js** | Web/PWA UI (workers mobile-first; supervisors/admins desktop-first); BFF-light via Route Handlers only where edge concerns require it |
| **Rust + Axum** | Primary API and modular domain host; authn/authz enforcement; transactional writes; outbox |
| **Go** | Background workers: notification delivery, analytics ingest, file post-processing, maintenance jobs |
| **PostgreSQL** | Operational system of record; module schemas; outbox; FTS initially |
| **Redis** | Cache, rate limits, short-lived locks, ephemeral session aids — never permanent business state |
| **Temporal** | Durable workflows, timers, escalations, package assembly, expiry watches |
| **NATS** | Event bus for integration events between modules and toward workers |
| **Cloudflare R2** | Object storage for documents, signatures, exports, attachments |
| **ClickHouse** | Analytical store for high-volume metrics and executive/project insights |
| **Docker** | Local/dev and deployable image packaging |
| **GitHub Actions** | CI/CD, security scanning, migrations gates, image publish |
| **Cloudflare** | DNS, WAF, CDN, Zero Trust (as needed), R2, edge protection |
| **Vercel** | Host Next.js web/PWA |
| **Fly.io** | Host Rust API, Go workers, and colocated supporting sidecars as required |

---

## 4. System Context Diagram (C4 Level 1)

```text
                                ┌─────────────────────────────────────────┐
                                │           External Identity             │
                                │     (SSO / OIDC / IdP providers)        │
                                └──────────────────▲──────────────────────┘
                                                   │ OIDC / SAML
┌──────────────┐    HTTPS/PWA     ┌────────────────┴──────────────────┐
│ Field Worker │─────────────────►│                                  │
│ Supervisor   │                  │             PROVEN               │
│ Safety Coord │◄─────────────────│   Construction Compliance OS     │
│ PM / Admin   │   Web + API      │                                  │
│ Executive    │                  └───────┬───────────┬───────────────┘
└──────────────┘                          │           │
                                          │           │ SMTP / Push / Webhooks
                                          ▼           ▼
                               ┌──────────────┐  ┌────────────────────┐
                               │ Email / Push │  │ Customer SSO / IdP │
                               │  Providers   │  │   (enterprise)     │
                               └──────────────┘  └────────────────────┘

External actors also include: auditors (export packages), insurers/clients (indirect via exports),
future ERP/HRIS/LMS systems (via governed APIs / ACL).
```

### 4.1 External Actors

| Actor | Interaction |
| --- | --- |
| Workers / Operators | Mobile PWA; offline-capable compliance tasks |
| Supervisors / Safety / PMs / Admins | Desktop-first web; configuration, review, dashboards |
| Executives | Scorecards and readiness views |
| Identity Providers | SSO for enterprise tenants |
| Email/Push providers | Notification delivery |
| Auditors (human) | Consume evidence packages (not a live integration initially) |
| Future systems (ERP/HRIS/LMS) | Inbound/outbound via ACL APIs (later phase) |

### 4.2 Trust Boundary

Everything inside **Proven** (Vercel web, Fly.io API/workers, managed data plane) is the primary trust boundary. Cloudflare sits at the edge. Secrets and keys never leave the platform vault/secret store into clients beyond short-lived scoped tokens (e.g., R2 upload URLs).

---

## 5. Container Diagram (C4 Level 2)

```text
                         ┌──────────────────────────────────────────┐
                         │              Cloudflare Edge             │
                         │     DNS · WAF · CDN · Bot protection     │
                         └───────────────┬──────────────────────────┘
                                         │
              ┌──────────────────────────┼──────────────────────────┐
              │                          │                          │
              ▼                          ▼                          ▼
┌──────────────────────┐   ┌─────────────────────────┐   ┌────────────────────┐
│  Web Application     │   │   API Platform          │   │  Object Storage    │
│  Next.js (Vercel)    │──►│   Rust / Axum (Fly.io)  │──►│  Cloudflare R2     │
│  PWA + Admin UI      │   │   Modular Monolith      │   │  docs/signatures   │
└──────────────────────┘   └───────────┬─────────────┘   └────────────────────┘
                                       │
           ┌───────────────┬───────────┼────────────┬────────────────┐
           │               │           │            │                │
           ▼               ▼           ▼            ▼                ▼
┌────────────────┐ ┌────────────┐ ┌─────────┐ ┌──────────┐ ┌─────────────────┐
│  PostgreSQL    │ │   Redis    │ │  NATS   │ │ Temporal │ │   ClickHouse    │
│  (primary SoR) │ │  (cache)   │ │ (events)│ │ (durable │ │   (analytics)   │
│  module schemas│ │            │ │         │ │  WF)     │ │                 │
└────────────────┘ └────────────┘ └────┬────┘ └────┬─────┘ └────────▲────────┘
                                       │           │                │
                                       ▼           ▼                │
                              ┌─────────────────────────────────────┴────────┐
                              │           Worker Fleet (Go) on Fly.io        │
                              │  notifications · analytics ETL · media jobs  │
                              └──────────────────────────────────────────────┘
```

### 5.1 Container Responsibilities

| Container | Responsibility | Does not |
| --- | --- | --- |
| **Next.js (Vercel)** | UI, PWA shell, offline client store, TanStack Query cache, form UX validation | Own business invariants; durable workflow state |
| **Rust/Axum API (Fly.io)** | Authn/authz, domain commands/queries, transactions, outbox, Temporal client starts/signals, presigned R2 flows | Long-running delivery loops; heavy analytical scans |
| **Go Workers (Fly.io)** | Consume jobs/events; send email/push; ClickHouse ingest; virus scan/transcode hooks; retries for I/O | Decide compliance outcomes; mutate domain rules |
| **PostgreSQL** | OLTP truth; outbox; audit; FTS | Hot analytics over multi-year event floods |
| **Redis** | Cache, rate limit, ephemeral coordination | Permanent storage of records/evidence |
| **NATS** | Fan-out integration events; worker triggers | Exactly-once business ledger (app idempotency required) |
| **Temporal** | Durable orchestration, timers, escalations | Domain invariant enforcement |
| **R2** | Binary objects | Queryable business state |
| **ClickHouse** | Analytical projections & aggregates | System of record for compliance entities |
| **Cloudflare** | Edge security, DNS, CDN, R2 access patterns | Application domain logic |
| **GitHub Actions** | Build, test, scan, migrate, deploy | Runtime business processing |

### 5.2 Synchronous vs Asynchronous Paths

| Path | Mechanism | Examples |
| --- | --- | --- |
| Sync request/response | HTTPS → Axum | Create activity, query eligibility, fetch project |
| Async domain fan-out | Postgres outbox → NATS | `SafetyActivitySubmitted` → COR, Analytics, Notifications |
| Durable multi-step | Temporal workflows + activities back into Axum public APIs | Escalations, multi-signer flows, COR package build |
| Background I/O | Go workers | Email send, CH insert, object post-process |

---

## 6. Component Diagram (C4 Level 3) — API Platform

The Rust/Axum process is a **modular monolith host** composed of domain modules plus platform components.

```text
┌──────────────────────────────────────────────────────────────────────────┐
│                        Axum Host (proven-api)                            │
│                                                                          │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────┐  ┌─────────────────┐│
│  │ HTTP Layer  │  │ AuthN Gate   │  │ AuthZ Gate │  │ Request Context ││
│  │ routing     │─►│ session/JWT  │─►│ RBAC scope │─►│ tenant/actor    ││
│  └─────────────┘  │ OIDC/SSO     │  │ project    │  │ correlation     ││
│                   └──────────────┘  └────────────┘  └─────────────────┘│
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                     Domain Modules (components)                    │ │
│  │  tenancy · identity · projects · workforce · safety · equipment    │ │
│  │  documents · signatures · training · cor_audit · notifications     │ │
│  │  workflows · analytics · audit                                     │ │
│  │                                                                    │ │
│  │  Each module: Application API │ Domain │ Infra Adapters            │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌──────────────┐ ┌─────────────┐ ┌─────────────┐ ┌──────────────────┐ │
│  │ Outbox        │ │ NATS Pub    │ │ Temporal    │ │ R2 Presign       │ │
│  │ Publisher     │ │ Adapter     │ │ Client      │ │ Adapter          │ │
│  └──────────────┘ └─────────────┘ └─────────────┘ └──────────────────┘ │
│  ┌──────────────┐ ┌─────────────┐ ┌─────────────┐ ┌──────────────────┐ │
│  │ Redis Cache   │ │ Postgres    │ │ Idempotency │ │ Observability    │ │
│  │ Adapter       │ │ Unit of Work│ │ Store       │ │ metrics/traces   │ │
│  └──────────────┘ └─────────────┘ └─────────────┘ └──────────────────┘ │
└──────────────────────────────────────────────────────────────────────────┘
```

### 6.1 Per-Module Internal Components

Every domain module follows the same internal shape:

| Component | Responsibility |
| --- | --- |
| **HTTP adapters** | Route handlers mapped to application commands/queries |
| **Application services** | Use-case orchestration within the module; call other modules only via public interfaces |
| **Domain model** | Aggregates, invariants, domain events |
| **Repository adapters** | PostgreSQL persistence for owned schema |
| **Policy hooks** | Emit audit entries; register outbox events; start/signal workflows when required |

### 6.2 Web Application Components (Next.js)

| Component | Responsibility |
| --- | --- |
| **App Router UI** | Role-appropriate layouts (worker vs admin) |
| **PWA runtime** | Installability, service worker, background sync hooks |
| **Offline mutation queue** | Durable client queue with mutation IDs |
| **TanStack Query** | Server state cache; revalidation |
| **Form layer** | RHF + Zod for input shaping (not business authority) |
| **Auth client** | Session bootstrap, token refresh, SSO redirects |
| **Upload client** | Presigned R2 upload with progress and checksum |

### 6.3 Worker Components (Go)

| Component | Responsibility |
| --- | --- |
| **NATS consumers** | Subscribe to integration subjects / work queues |
| **Temporal workers** (optional split) | Host activities that are I/O heavy if not run in Rust |
| **Notification dispatchers** | Email/push/SMS providers with retry/backoff |
| **Analytics pipeline** | Transform events → ClickHouse inserts |
| **Object jobs** | Post-upload processing (checksum verify, AV scan hook, thumbnail if needed) |
| **Dead-letter handling** | Isolate poison messages; alert |

> Prefer **Rust Temporal activities** that call in-process module APIs for domain commands. Use **Go** for provider I/O and ETL. If Temporal workers are split by language, keep activity contracts stable and thin.

---

## 7. Module Relationships

Aligned to bounded contexts from the Domain Model.

### 7.1 Relationship Matrix

| From → To | Interface | Events | Workflows |
| --- | --- | --- | --- |
| **identity** → all | AuthZ decisions at gate | Access/session security events | Step-up / revoke flows (rare) |
| **tenancy** → all | Tenant/company validation | Entitlement changes | Tenant provisioning |
| **projects** → safety/training/equipment/documents | Membership & required control queries | Project lifecycle events | Project onboarding |
| **workforce** → safety/training/projects | Person/crew queries | Person lifecycle events | — |
| **safety** → signatures/documents/equipment/training | Create signature pkgs; resolve docs; readiness/eligibility queries | Activity/action events | Review & escalation |
| **equipment** → documents/signatures/projects | Cert/doc refs; sign-offs | Readiness/expiry events | Inspection due / expiry |
| **documents** → signatures | Ack + sign | Version/ack events | Distribution acknowledgement |
| **training** → documents/signatures | Evidence attachments | Competency/expiry events | Expiry & reminders |
| **cor_audit** ← many | Evidence queries (read) | Consumes compliance facts | Evidence package generation |
| **notifications** ← many | — | Consumes notifiable events | Digest scheduling |
| **workflows** → many | Starts commands via public APIs | Workflow lifecycle events | Orchestrates all durable processes |
| **analytics** ← many | — | Projection consumers | Rebuild jobs |
| **audit** ← all | Append API | Optional fan-out | Export workflows |

### 7.2 Dependency Direction Rules

```text
UI → HTTP API → AuthN/AuthZ → Owning Module
Owning Module → (optional) Public Interface of other Module
Owning Module → Outbox → NATS → Downstream Modules / Workers
Owning Module / API → Temporal (start/signal)
Temporal Activities → Public Module APIs only
Go Workers → Providers / ClickHouse / R2 (I/O); may call internal worker HTTP for status only
```

**Forbidden:** module A SQL-joining module B tables; UI calling Temporal directly; workers writing directly to another module’s tables.

### 7.3 Published Contracts

| Contract type | Owner | Consumers |
| --- | --- | --- |
| HTTP OpenAPI per module surface | API platform | Next.js, partners (later) |
| Public application interfaces (in-process) | Owning module | Other modules, Temporal activities |
| NATS event schemas (versioned) | Owning module | notifications, analytics, cor_audit, caches |
| Temporal workflow/activity contracts | workflows module + owning domains | API, workers |

---

## 8. Workflow Engine

### 8.1 Role of Temporal

Temporal is the **system of process** for durable business workflows:

- Assign → complete → review → close
- Multi-party signature sequencing
- Corrective action SLA / escalation
- Training and certification expiry watches
- COR evidence package assembly
- Tenant/project onboarding checklists
- Notification digests that require durable timers

Temporal is **not** the system of record for Safety/Training/Equipment entities. Aggregates remain in PostgreSQL modules.

### 8.2 Logical Workflow Architecture

```text
API Command
   │
   ├─► Persist aggregate (Postgres)
   ├─► Outbox domain event
   └─► Start / signal Temporal workflow (when process needed)
            │
            ▼
     Temporal Server
            │
            ├─► Activity: call Axum public command/query
            ├─► Timer / retry / compensation
            ├─► Activity: enqueue notification intent
            └─► Activity: update workflow visibility projection
```

### 8.3 Workflow Ownership

| Concern | Owner |
| --- | --- |
| Workflow definitions (template metadata, tenant config) | `workflows` module |
| Temporal workflow implementations | Platform workflows package |
| Domain decisions inside activities | Owning domain module APIs |
| Timers & escalations | Temporal |
| User-visible status (“where is this?”) | Projection from workflow events + domain status |

### 8.4 Reliability Rules

- Activities are **idempotent** (keyed by workflow ID + business mutation ID).
- Heartbeats for long package builds.
- Hard timeouts aligned to SLA policies.
- Cancel/void paths must leave domain aggregates in a legal terminal state.
- Poison activities → alert + manual intervention playbooks; never silent drop of compliance process.

---

## 9. Notification System

### 9.1 Pipeline

```text
Domain Event (NATS)
        │
        ▼
notifications module (Axum)
  - evaluate DeliveryRule
  - apply Preference / quiet hours
  - deduplicate (DedupKey)
  - persist Notification record
  - enqueue delivery job
        │
        ▼
Go Notification Worker
  - render template
  - send via Email / Push / SMS providers
  - record DeliveryAttempt
  - retry with backoff / DLQ
```

### 9.2 Channels

| Channel | Priority use | Notes |
| --- | --- | --- |
| In-app | Default for all users | Stored in Postgres; realtime via poll or push bridge |
| Email | Approvals, expiries, digests | Provider via worker |
| Push (PWA) | Field-critical assignments | Where browser/platform supports |
| SMS | Opt-in critical escalations only | Regional compliance constraints |

### 9.3 Design Rules

- **Notification content is derived from events**, not recomputed business rules in Go.
- Workers may choose provider routing; they may not decide whether a corrective action is overdue.
- Dedup keys prevent double-send on at-least-once NATS delivery.
- Tenant policies can force-critical channels (e.g., safety escalation cannot be fully muted).

---

## 10. Storage Architecture

### 10.1 Storage Map

| Data class | Store | Notes |
| --- | --- | --- |
| Operational entities / aggregates | **PostgreSQL** | Schema-per-module |
| Outbox & idempotency keys | **PostgreSQL** | Same DB, module or platform schemas |
| Audit entries | **PostgreSQL** (append-only) | Exportable; integrity digests |
| Sessions / rate limits / hot caches | **Redis** | TTL-bound; disposable |
| Documents, signatures, exports, attachments | **Cloudflare R2** | Object keys + checksums in Postgres |
| Analytical facts / rollups | **ClickHouse** | Fed by workers from events |
| Client offline drafts | **Browser storage (PWA)** | Not authoritative until sync |

### 10.2 PostgreSQL Layout

- **One logical database** initially (managed Postgres).
- **Schema per module** (`safety`, `training`, `projects`, …) plus `platform` (outbox publisher locks, migration ledger).
- Cross-module references stored as **UUIDs**, not FK across schemas (optional soft discipline via application).
- Migrations owned per module; CI blocks violating cross-schema writes in module code.

### 10.3 Object Storage (R2)

```text
Client → API: request upload intent
API → authorize + create PendingObject metadata
API → return presigned PUT URL (short TTL, content-type/size constrained)
Client → PUT object to R2
Client → API: complete upload (checksum)
API → validate → mark object Available → domain attach
```

Download: authorized **presigned GET** or authenticated streaming gateway for sensitive evidence.

Buckets/prefixes separated by tenant and class (`documents/`, `signatures/`, `exports/`). Object versioning and lifecycle policies per class.

### 10.4 ClickHouse

- Receives **denormalized analytical events** from Go analytics workers.
- Used for portfolio trends, hotspots, large scan dashboards.
- Not queried for authoritative eligibility decisions.
- Rebuildable from Postgres/event history for defined windows when required.

### 10.5 Redis Usage (Allowed vs Forbidden)

| Allowed | Forbidden |
| --- | --- |
| HTTP response cache for safe read models | Storing compliance records as truth |
| Rate limiting / abuse counters | Permanent session-of-record without Postgres backing |
| Short-lived distributed locks | Evidence / signature blobs |
| Idempotency assist (optional; Postgres wins) | “Temporary” data without TTL and rebuild plan |

---

## 11. Authentication

### 11.1 Mechanisms

| Actor type | Mechanism |
| --- | --- |
| Standard users | Email/password or magic-link (as product dictates) + session |
| Enterprise users | **OIDC/SAML SSO** via customer IdP |
| Workers (PWA) | Same identity; refresh-friendly session cookies or rotating tokens |
| Service/workers | Mutual service credentials (Fly secrets); no user impersonation without explicit audited reason |
| Object upload | Presigned URLs derived from authenticated API intent |

### 11.2 Session Model

- Authentication established at **Identity** module.
- Server-side session authority in Postgres (revocable); Redis may cache session validation with short TTL.
- Tokens/cookies: secure, HTTP-only, SameSite appropriate to web topology; CSRF protections for cookie sessions.
- SSO: map external subject → `Principal` → `PersonId` within tenant.

### 11.3 Step-Up & Revocation

- Sensitive actions (export packages, role changes, mass distribution) may require recent authentication or SSO reauth.
- Admin can revoke sessions; Identity emits security events; Audit records reason.

---

## 12. Authorization

### 12.1 Model

**RBAC with scoped grants**:

```text
Principal + Role + Scope(Tenant | OrgUnit | Project) + PermissionCode
```

Authorization is enforced in the **API AuthZ gate** and re-checked inside modules for defense in depth on sensitive aggregates.

### 12.2 Scope Rules

| Scope | Typical use |
| --- | --- |
| Tenant | Company admins, safety program owners |
| OrgUnit | Regional managers |
| Project | PMs, supervisors, workers on that project |
| Self | Worker can read/update own limited profile & assignments |

GC/Sub visibility is controlled by **project participation** + least-privilege roles—not by sharing tenant-wide admin.

### 12.3 Decision Flow

```text
Request
  → authenticate Principal
  → resolve Tenant
  → load grants (cached in Redis, invalidated on Access* events)
  → evaluate permission + resource scope (project/company)
  → allow/deny (deny audited for sensitive resources)
  → module command
```

### 12.4 Policy Ownership

- Permission catalog owned by Identity.
- Resource membership facts owned by Projects/Workforce/etc.
- AuthZ gate composes Identity grants with resource scope queries (public interfaces)—**no cross-schema joins**.

---

## 13. Offline Sync

### 13.1 Goals

Field workers complete priority flows with intermittent connectivity and sync without duplicating or corrupting aggregates.

### 13.2 Client Responsibilities (Next.js PWA)

- Maintain an **offline mutation queue** with client-generated `mutation_id` (UUIDv7/ULID).
- Store drafts and reference data snapshots needed for assigned tasks.
- Surface sync state: pending, syncing, conflict, failed.
- Never invent server authority (eligibility may be last-known snapshot with explicit staleness UX).

### 13.3 Server Responsibilities (Axum modules)

- Accept idempotent commands keyed by `(principal/tenant, mutation_id)`.
- Validate invariants on sync (reject illegal state transitions).
- Return canonical resource state after apply.
- Bound which aggregates/types are offline-writable (allowlist per module).

### 13.4 Sync Protocol (Logical)

```text
1. Client enqueues mutation (mutation_id, type, payload, base_version?)
2. On reconnect, drain queue in causal order per aggregate where required
3. API applies or rejects
4. If conflict: domain-defined resolution (reject / merge-safe fields / require user action)
5. Client replaces draft with server canonical state
6. Domain events + audit emitted only on successful apply
```

### 13.5 Conflict Policy Examples

| Case | Policy |
| --- | --- |
| Duplicate mutation_id | Return original success result (idempotent) |
| Activity already closed server-side | Reject offline close/edit; prompt refresh |
| Concurrent draft edits | Last safe write or field-level merge only where domain allows |
| Signature already completed | Reject additional capture |

### 13.6 Offline Scope (Initial)

**In:** safety activity drafts/submits, acknowledgements, pre-use inspections, training evidence upload intents metadata.  
**Out (initially):** tenant admin, role changes, COR package generation, bulk distribution.

---

## 14. Caching

### 14.1 Layers

| Layer | Technology | Contents |
| --- | --- | --- |
| Edge CDN | Cloudflare | Static Next assets; cacheable public marketing only |
| Client | TanStack Query + PWA cache | UI server state; offline reference data |
| API | Redis | AuthZ grant cache, hot read DTOs, rate limits |
| DB | PostgreSQL | Authoritative; materialized read models where needed |

### 14.2 Cache Rules

- Cache **read models and decisions**, not aggregates as mutable truth.
- Invalidate on relevant NATS events (`AccessRevoked`, `ProjectMembership*`, `DocumentVersionPublished`, etc.).
- TTLs always present; stale-while-revalidate acceptable for non-safety-critical displays only.
- Eligibility decisions used for **enforcement** must be fresh or explicitly bounded; do not rely on long-lived Redis eligibility alone.

### 14.3 Rate Limiting

Redis-backed limits at API edge (per principal, IP, tenant) to protect write endpoints and auth routes. Cloudflare WAF provides additional edge throttling.

---

## 15. Deployment Architecture

### 15.1 Environment Topology

| Environment | Web | API / Workers | Data |
| --- | --- | --- | --- |
| Local | Docker Compose / Next dev | Dockerized API, workers, Temporalite/NATS/Redis/Postgres | Local volumes |
| Preview | Vercel Preview | Ephemeral/staging Fly app (or shared staging) | Staging DB isolated |
| Staging | Vercel staging project | Fly staging | Staging Postgres/Redis/NATS/Temporal/R2/CH |
| Production | Vercel production | Fly production (multi-region as needed) | HA managed data plane |

### 15.2 Runtime Placement

```text
Users
  → Cloudflare (DNS/WAF/CDN)
    → Vercel (Next.js)
      → Fly.io (proven-api, proven-workers)
        → Managed PostgreSQL
        → Managed Redis
        → NATS cluster
        → Temporal Cloud or self-hosted Temporal on Fly
        → Cloudflare R2
        → ClickHouse Cloud or self-hosted
```

### 15.3 CI/CD (GitHub Actions)

Pipeline stages:

1. **Change detection** — module-aware builds where practical  
2. **Lint / typecheck / unit / domain tests**  
3. **Security scans** — deps, SAST, container scan  
4. **Build images** — API (Rust), workers (Go)  
5. **Migrate** — Postgres migrations (gated, forward-only preferred)  
6. **Deploy** — Vercel (web), Fly (API/workers)  
7. **Smoke / synthetic checks**  
8. **Release annotations** — version, SBOM artifact retention  

### 15.4 Release Strategy

- Web: Vercel atomic deployments with instant rollback.
- API/Workers: Fly rolling deploys with health checks; blue/green optional for major versions.
- Migrations: expand/contract pattern; never lock for long; backward-compatible API during rollouts.
- Feature flags / module entitlements for progressive exposure.

### 15.5 Configuration & Secrets

- Secrets in GitHub Actions + Fly secrets / Vercel env; never in git.
- Environment-specific config for NATS subjects, Temporal namespace, R2 buckets, IdP clients.
- Separate R2 buckets and DB instances per environment.

---

## 16. Security Architecture

### 16.1 Security Principles

- Security first; least privilege; deny by default.
- Tenant isolation on every query path.
- Encryption in transit (TLS everywhere); encryption at rest for Postgres, Redis, R2, ClickHouse as offered by providers + application-level controls for sensitive exports.
- Audit all admin, authz changes, signatures, closures, exports.
- No business permanent data in Redis.
- Supply chain security via CI scanning and pinned base images.

### 16.2 Edge & Network

- Cloudflare WAF, DDoS, bot management.
- Restrict administrative endpoints; optional Cloudflare Access / Zero Trust for staging and admin routes.
- Private connectivity patterns for database access from Fly (provider-recommended).
- Minimal public surface: Web, API, OIDC callbacks, presigned R2 URLs.

### 16.3 Application Security

- AuthN required for all non-public routes.
- AuthZ scope checks on every resource.
- Input validation at API boundary; domain validation in aggregates.
- Idempotency keys on writes to reduce replay abuse.
- Strict CORS; security headers on web.
- File uploads: type/size limits, checksum verification, malware scanning hook in workers, quarantine prefix until clean.

### 16.4 Data Security

- Tenant ID mandatory on rows and object keys.
- Presigned URLs short-lived and action-scoped.
- PII minimization in events and analytics (identifiers preferred; hash/tokenizeize where possible).
- Retention and legal hold handled in Documents/Audit policies.
- Secrets rotation procedures documented for IdP, DB, R2, provider API keys.

### 16.5 Secure SDLC

- PR reviews; protected main branch.
- Dependency vulnerability gates.
- Infrastructure-as-code review for Fly/Cloudflare/Vercel config.
- Periodic access reviews for production break-glass accounts.

---

## 17. Disaster Recovery

### 17.1 Objectives (Initial Targets)

| Capability | RPO | RTO | Notes |
| --- | --- | --- | --- |
| PostgreSQL (SoR) | ≤ 5–15 min | ≤ 1 hour | Provider PITR + replicas |
| R2 objects | ≤ 24 h (versioning/replication strategy) | ≤ 4 hours | Object versioning + backup plan |
| Temporal | Provider SLA | ≤ 1 hour | Workflows resume; activities idempotent |
| NATS | Minutes | Minutes–1 hour | Rebuild consumers; durable intent in outbox |
| Redis | Acceptable full loss | Minutes | Rebuild from Postgres |
| ClickHouse | ≤ 24 h | ≤ 8 hours | Re-ingest from events/outbox archive |
| Vercel/Fly compute | N/A (stateless) | Minutes | Redeploy from images |

Exact contractual numbers finalized per customer tier.

### 17.2 Backup & Restore

- **Postgres:** continuous backup / PITR; encrypted backups; quarterly restore drills.
- **R2:** versioning on evidence buckets; cross-bucket replication for critical prefixes if required.
- **ClickHouse:** periodic snapshots + rebuild pipelines.
- **Temporal:** namespace backup per provider guidance.
- **Secrets:** redundant secret storage with controlled access.

### 17.3 Failure Modes & Responses

| Failure | Response |
| --- | --- |
| Single API instance down | Fly health checks replace instance |
| Region impairment | Fail over Fly region + DNS; read-only mode if DB failover incomplete |
| NATS outage | Outbox buffers in Postgres; replay when restored |
| Temporal outage | New durable starts queue; running workflows pause/resume; critical manual ops playbook |
| Redis outage | Degrade cache; auth may hit Postgres directly; raise latency alerts |
| R2 outage | Block new uploads; allow metadata reads; queue object intents |
| Vercel outage | Status comms; API remains for mobile if alternative shell needed (future) |
| Corrupt deploy | Instant rollback web; Fly previous release; migrate forward-fix if needed |

### 17.4 Data Integrity During DR

- Prefer **restore + replay outbox** over hand-editing compliance rows.
- Evidence packages and signatures rely on object versioning + Postgres metadata consistency checks after restore.
- Post-incident: audit trail of administrative recovery actions.

---

## 18. Scalability

### 18.1 Scale Dimensions

| Dimension | Strategy |
| --- | --- |
| Web traffic | Cloudflare CDN + Vercel autoscaling |
| API throughput | Horizontal Fly machines for Axum; stateless API tier |
| Write intensity | Postgres primary scaling + connection pooling; module-friendly schema |
| Event fan-out | NATS horizontal scale; consumer groups per worker type |
| Workflow volume | Temporal worker autoscaling; activity concurrency limits |
| Notification spikes | Go worker queues with backoff; digests to collapse storms |
| Analytics | ClickHouse scale separation from OLTP |
| Object bandwidth | R2 + presigned direct upload/download |
| Multi-tenant growth | Tenant-based rate limits; entitlement-gated modules |

### 18.2 Modular Monolith Scaling Path

**Phase A — Vertical/horizontal monolith**  
Scale `proven-api` replicas; shared DB with pooling.

**Phase B — Worker isolation**  
Independent scaling of Go notification vs analytics fleets.

**Phase C — Read path optimization**  
Postgres read replicas for heavy queries; Redis for hot DTO cache; ClickHouse for dashboards.

**Phase D — Selective extraction**  
Extract a module (e.g., notifications or analytics ingest) only when:  
stable public contracts exist, independent scale pain is proven, and operational maturity justifies distributed cost.

### 18.3 Performance Budgets (Architectural)

- Common API reads: p95 < 300 ms server-side under normal load (excludes large transfers).
- Offline sync drain: bounded batch sizes; fairness per tenant.
- Dashboard heavy queries: served from ClickHouse/read models, not deep OLTP joins.
- Upload/download: direct to R2; API handles intent and completion only.

### 18.4 Multi-Region Considerations

- Primary write region for Postgres initially; evaluate multi-region read for CA/US/AU/NZ as GTM expands.
- Keep Temporal and API near the data primary to avoid cross-region write latency.
- Edge web remains global via Cloudflare/Vercel.
- Data residency requirements may force regional deployments later; module boundaries and tenant region metadata make this feasible without redesigning domains.

### 18.5 Backpressure & Load Shedding

- Rate limits at Cloudflare and API.
- Queue depth alerts for workers and Temporal task queues.
- Degrade non-critical features first (analytics freshness, digests) before blocking safety writes.
- Safety/signature submits prioritized over reporting rebuilds.

---

## 19. Observability & Operability

| Signal | Practice |
| --- | --- |
| Logs | Structured JSON; tenant/actor/correlation IDs; PII redaction |
| Metrics | RED/USE for API; queue lag; workflow failure rates; sync failure rates |
| Traces | Trace request → module → DB/NATS/Temporal |
| Alerts | Error budget, DB saturation, outbox lag, worker DLQ, auth anomaly |
| Runbooks | Deploy, rollback, replay outbox, rotate secrets, restore DB |

---

## 20. End-to-End Reference Flows

### 20.1 Submit Safety Activity (Online)

```text
PWA → Cloudflare → Vercel (UI)
   → Fly Axum (AuthN/AuthZ)
   → safety module command
   → Postgres commit + outbox + audit
   → start/signal Temporal if review required
   → NATS publish SafetyActivitySubmitted
   → notifications / cor_audit / analytics consumers
```

### 20.2 Offline Inspection Sync

```text
PWA queue mutation
   → reconnect
   → Axum equipment.InspectionComplete (idempotent)
   → readiness update + events
   → notifications on failure
   → analytics/COR projections
```

### 20.3 COR Package Generation

```text
Admin request → cor_audit
   → Temporal package workflow
   → activities query evidence via public APIs
   → assemble manifest; write export to R2
   → seal EvidencePackage
   → notify requester; audit export
```

---

## 21. Architecture Decision Summary

| Decision | Choice | Rationale |
| --- | --- | --- |
| Application shape | Modular monolith (Rust) | Speed + strong boundaries; extract later |
| UI | Next.js PWA on Vercel | Mobile/desktop surfaces; edge deploy |
| Process | Temporal | Durable compliance processes |
| Events | NATS + Postgres outbox | Reliable integration without dual-writes |
| Workers | Go | Efficient I/O for delivery/ETL |
| OLTP | PostgreSQL | SoR, FTS initially, strong consistency |
| Cache | Redis | Performance only |
| Objects | R2 | Cost-effective evidence storage at edge-friendly provider |
| Analytics | ClickHouse | Heavy read/aggregate isolation |
| Edge | Cloudflare | Security + performance |
| Compute API | Fly.io | Global VMs close to data plane |

---

## 22. Out of Scope for This Document

- Detailed ERDs / SQL DDL  
- OpenAPI schemas  
- Exact Terraform/Fly.toml listings  
- Vendor contract pricing  
- Per-customer residency legal opinions  

Those belong in subsequent design specs derived from this architecture and the Domain Model.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Principal Architecture | Initial complete system architecture for Proven |

---

*End of System Architecture*
