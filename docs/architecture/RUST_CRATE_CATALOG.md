# Proven — Rust Crate Catalog

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Rust Crate Design Catalog |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Lead Rust Engineering |
| **Audience** | Backend Engineering, Module Owners, Platform |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Rust Backend Architecture](./RUST_BACKEND_ARCHITECTURE.md), [Repository Plan](./REPOSITORY_PLAN.md), [Event Catalog](./EVENT_CATALOG.md), [PostgreSQL](./POSTGRESQL_ARCHITECTURE.md), domain docs, [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document designs **every Rust crate** in the Proven workspace: supporting platform crates and each domain module crate (**Core, Projects, People, Safety, Equipment, Documents, Signatures, Notifications, Training, COR, Analytics, Administration, Integrations**).

For each crate: Purpose, Public API, Dependencies, Events, Database, Configuration, Testing, Folder Structure, Ownership.

**Documentation only — no implementation.**

---

## 2. Workspace Overview

```text
Cargo workspace
├── apps/api                         # binary: proven-api (thin main)
└── crates/
    ├── proven-shared
    ├── proven-platform
    ├── proven-contracts             # optional shared DTO mirrors
    ├── proven-test-support
    └── modules/
        ├── proven-core
        ├── proven-projects
        ├── proven-people
        ├── proven-safety
        ├── proven-equipment
        ├── proven-documents
        ├── proven-signatures
        ├── proven-notifications
        ├── proven-training
        ├── proven-cor
        ├── proven-analytics
        ├── proven-admin
        ├── proven-integrations
        └── proven-workflows         # Temporal orchestration ports (domain-adjacent)
```

### 2.1 Dependency Rules

```text
apps/api → proven-platform → registers each module
module → proven-shared
module → other modules ONLY via public trait APIs (no SQL/schema imports)
proven-platform → infra adapters (PgPool, NATS, Temporal, Redis, R2)
```

**Forbidden:** cross-module `infrastructure`/`sql` imports; business rules in handlers only; Go/React owning invariants.

### 2.2 Standard Module Folder Structure

Every `proven-<module>` follows:

```text
crates/modules/proven-<module>/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs                 # public exports: ModuleApi traits, router fn
│   ├── domain/                # aggregates, VOs, invariants
│   ├── application/           # commands, queries, ports (traits)
│   ├── infrastructure/        # SQLx repos, outbox writers, adapters
│   ├── api/                   # HTTP handlers, DTOs, route registration
│   ├── events/                # event payload types + mappers
│   └── config.rs              # module config section
└── tests/                     # integ tests (optional; prefer src + test-support)
```

---

## 3. Supporting Crates

### 3.1 `proven-shared`

| Aspect | Design |
| --- | --- |
| **Purpose** | Shared kernel: typed IDs, time, correlation, paging, error skeleton, problem-details mapping helpers. **No business rules.** |
| **Public API** | `TenantId`, `UserId`, `PersonId`, `ProjectId`, `FileObjectId`, …; `CorrelationId`; `PageCursor`; `AppError` / `ErrorCode`; `Instant` wrappers |
| **Dependencies** | `uuid`, `serde`, `thiserror`, `chrono`/`time` — **no** SQLx, Axum, module crates |
| **Events** | None owned |
| **Database** | None |
| **Configuration** | None |
| **Testing** | Unit tests for ID parsing/display; proptests optional |
| **Folder Structure** | `src/{ids,error,paging,time,correlation}.rs` |
| **Ownership** | Platform / backend staff |

---

### 3.2 `proven-platform`

| Aspect | Design |
| --- | --- |
| **Purpose** | Composition root: Axum router merge, middleware (authn, tenant RLS GUC, request-id, timeout), `AppState`, config load, outbox publisher loop, health/ready, Temporal/NATS/Redis/R2 clients. |
| **Public API** | `build_app(config) -> Router`; `AppState`; middleware stack; infra ports re-export for wiring |
| **Dependencies** | All module crates (wiring only); `proven-shared`; `axum`, `tower`, `tokio`, `sqlx`, NATS/Temporal/Redis/R2 clients, tracing |
| **Events** | Publishes outbox → NATS (transport); does not define domain events |
| **Database** | Pool + session setup; no domain schemas |
| **Configuration** | Root config: `DATABASE_URL`, NATS, Temporal, Redis, R2, HTTP bind, env |
| **Testing** | Smoke boot with testcontainers; middleware unit tests |
| **Folder Structure** | `src/{app,middleware,state,config,outbox,health,infra}.rs` |
| **Ownership** | Platform / backend staff |

---

### 3.3 `proven-contracts` (optional)

| Aspect | Design |
| --- | --- |
| **Purpose** | Versioned OpenAPI/event type mirrors shared with codegen if not generated into `packages/api-client` alone. |
| **Public API** | Serde DTOs aligned to `contracts/` |
| **Dependencies** | `proven-shared`, `serde` |
| **Events / DB / Config** | Mirrors only |
| **Testing** | Schema round-trip |
| **Ownership** | Backend + contracts reviewers |

---

### 3.4 `proven-test-support`

| Aspect | Design |
| --- | --- |
| **Purpose** | Testcontainers Postgres, tenant bootstrap helpers, auth principal fixtures, assert helpers. |
| **Public API** | `TestDb`, `with_tenant`, `principal_fixture` |
| **Dependencies** | `sqlx`, testcontainers, `proven-shared`, optionally `proven-core` for bootstrap |
| **Ownership** | Platform |

---

### 3.5 `proven-workflows`

| Aspect | Design |
| --- | --- |
| **Purpose** | Temporal client ports, workflow start/signal helpers, instance projection APIs used by modules; **not** a place for domain invariants. Catalog alignment with [TEMPORAL_WORKFLOWS.md](./TEMPORAL_WORKFLOWS.md). |
| **Public API** | `WorkflowPort` (`start`, `signal`, `cancel`, `describe`); HTTP visibility routes under `/workflows` |
| **Dependencies** | `proven-shared`, `proven-core` (AuthZ); Temporal SDK; modules call port—not Temporal directly from domain |
| **Events** | `WorkflowStarted/Completed/Failed`, `EscalationTriggered` |
| **Database** | Schema `workflows` — definitions/instances tracking |
| **Configuration** | Temporal host, namespace, task queues |
| **Testing** | Port fakes; integ with Temporal test server optional |
| **Folder Structure** | Standard module layout |
| **Ownership** | Platform + workflow owners |

---

## 4. Domain Module Crates

---

### 4.1 `proven-core`

| Aspect | Design |
| --- | --- |
| **Purpose** | Platform foundation: tenancy, companies/orgs, identity, sessions, RBAC/ABAC AuthZ, project membership, teams, file objects, audit, settings, feature flags, licensing. |
| **Public API** | `AuthzApi` (`authorize`, `list_scopes`); `TenantApi`; `IdentityApi` / session; `MembershipApi`; `FileApi` (`create_upload_intent`, `complete`, `authorize_file_access`); `AuditApi` (`append`); `SettingsApi`; `LicenseApi`; `FeatureFlagApi`; HTTP `/auth/*`, `/tenants`, `/companies`, `/org-units`, `/users`, `/roles`, `/memberships`, `/teams`, `/files`, `/audit`, `/settings`, `/licenses` |
| **Dependencies** | `proven-shared`; **no** other domain modules. Consumed by all modules via traits. |
| **Events** | `Tenant*`, `Company*`, `OrgUnit*`, `User*`, `Session*`, `AccessGranted/Revoked`, `ProjectMembership*`, `Team*`, `FileObject*`, `License*`, `FeatureFlag*`, `AuditEntryAppended` (optional bus) |
| **Database** | Schema `core` — tenants, users, credentials/SSO links, sessions, roles, grants, memberships, teams, file_objects, audit_entries, settings, flags, licenses |
| **Configuration** | JWT/OIDC, session TTL, MFA policy defaults, R2 bucket prefixes for files, password policy knobs |
| **Testing** | AuthZ matrix (IDOR); RLS tenant isolation; session revoke; file intent lifecycle |
| **Folder Structure** | Standard (+ `domain/authz`, `domain/files`, `domain/audit`) |
| **Ownership** | `@proven-backend` + `@proven-security` |

Domain doc: [CORE_DOMAIN.md](./CORE_DOMAIN.md).

---

### 4.2 `proven-projects`

| Aspect | Design |
| --- | --- |
| **Purpose** | Construction **Place** lifecycle: projects, areas, participants, templates, required controls, proof-health projections (operational). |
| **Public API** | `ProjectApi` (create/update/activate/archive); `ParticipantApi`; `ProjectTemplateApi`; queries for project dashboard; HTTP `/projects`, `/projects/{id}/areas`, `/participants`, `/templates` |
| **Dependencies** | `proven-shared`, `proven-core` (AuthZ, membership exists as binding—Projects owns Place; membership ACL in Core); optional query ports to People/Safety for dashboard composition via traits—not SQL |
| **Events** | `ProjectCreated/Updated/Activated/Archived`, `ProjectParticipant*`, `ProjectProofHealthChanged`, `RequiredControl*` |
| **Database** | Schema `projects` — projects, areas, participants, templates, required_controls, proof_health_snapshots |
| **Configuration** | Default template ids; proof-health weight hooks (references metric keys, not CH) |
| **Testing** | Lifecycle transitions; participant rules; activate seeds (via workflow/events) |
| **Folder Structure** | Standard |
| **Ownership** | `@proven-backend` (projects owners) |

Domain doc: [PROJECTS_DOMAIN.md](./PROJECTS_DOMAIN.md).

---

### 4.3 `proven-people`

| Aspect | Design |
| --- | --- |
| **Purpose** | Workforce **Person** profiles, trades, workforce roles, employment/contractor engagements, attendance, fit-for-work **signals** (no clinical PHI stores), certification profile refs. |
| **Public API** | `PersonApi` (register/update/activate); `TradeApi`; `EmploymentApi`; `AttendanceApi`; `FitSignalApi`; HTTP `/workers`, `/people`, trades, attendance |
| **Dependencies** | `proven-shared`, `proven-core` (AuthZ; User↔Person link); **not** Training SQL—Training consumes PersonId events |
| **Events** | `PersonRegistered/Updated/Activated/Deactivated/Archived`, `TradeAssigned/Removed`, `WorkforceRole*`, `Employment*`, `ContractorEngagement*`, `Attendance*`, `FitForWorkSignalChanged`, `CertificationProfileEntry*` |
| **Database** | Schema `people` — persons, trades, roles, employments, attendance, fit_signals, cert_profile_entries |
| **Configuration** | Attendance policies; PII field visibility defaults |
| **Testing** | PII minimization; activation; attendance void/correct; no medical note leakage in events |
| **Folder Structure** | Standard |
| **Ownership** | `@proven-backend` (people owners) |

Domain doc: [PEOPLE_DOMAIN.md](./PEOPLE_DOMAIN.md).

---

### 4.4 `proven-safety`

| Aspect | Design |
| --- | --- |
| **Purpose** | Safety activities (FLHA, toolbox, inspections), hazards/controls, corrective actions, incidents, near misses, bulletins, permits, lift plans; submit/review/close/void invariants. |
| **Public API** | `SafetyActivityApi`; `CorrectiveActionApi`; `IncidentApi`; `BulletinApi`; `PermitApi`; `LiftPlanApi`; HTTP `/safety/*` (flhas, activities, cas, incidents, …) |
| **Dependencies** | `proven-shared`, `proven-core` (AuthZ, files); ports: `SignaturesPort`, `ProjectsPort`, `DocumentsPort` (SWP refs), `WorkflowPort`; **no** Equipment/Training schema access |
| **Events** | `SafetyActivity*`, `CorrectiveAction*`, `Incident*`, `NearMiss*`, `Bulletin*`, `Permit*`, `LiftPlan*`, hazard library events |
| **Database** | Schema `safety` — activity_types, activities, hazards, controls, cas, incidents, bulletins, permits, lift_plans, photos refs |
| **Configuration** | Per-type offline allowlist flags; risk thresholds; signature policy refs |
| **Testing** | Submit invariants (hazards/controls); sealed immutability; CA SLA fields; offline idempotency keys |
| **Folder Structure** | Standard (+ `domain/activity`, `domain/ca`, `domain/incident`) |
| **Ownership** | `@proven-safety` + `@proven-backend` |

Domain doc: [SAFETY_DOMAIN.md](./SAFETY_DOMAIN.md).

---

### 4.5 `proven-equipment`

| Aspect | Design |
| --- | --- |
| **Purpose** | Assets, readiness, inspections (pre-use/periodic), certifications, deficiencies, binders, maintenance orders, OOS/release. |
| **Public API** | `AssetApi`; `ReadinessApi`; `InspectionApi`; `CertificationApi`; `DeficiencyApi`; `BinderApi`; `MaintenanceApi`; HTTP `/equipment/*` |
| **Dependencies** | `proven-shared`, `proven-core`; ports to Projects (assignment), Safety (optional activity link), Workflows; Files for attachments |
| **Events** | `Asset*`, `Inspection*`, `AssetReadinessChanged`, `Certification*`, `Deficiency*`, `BinderCompletenessChanged`, `MaintenanceOrder*`, `OutOfService*` |
| **Database** | Schema `equipment` — assets, inspections, certs, deficiencies, binders, maint_orders, readiness_state |
| **Configuration** | Validity windows; readiness rule toggles; binder section schemas |
| **Testing** | Readiness transitions; pre-use validity; binder completeness; OOS release gates |
| **Folder Structure** | Standard |
| **Ownership** | `@proven-equipment` + `@proven-backend` |

Domain doc: [EQUIPMENT_DOMAIN.md](./EQUIPMENT_DOMAIN.md).

---

### 4.6 `proven-documents`

| Aspect | Design |
| --- | --- |
| **Purpose** | Controlled documents, versions, approval/publish, acknowledgements, SWP/SJP meaning, QR targets, search projections hooks. |
| **Public API** | `DocumentApi`; `DocumentVersionApi`; `AcknowledgementApi`; `DocumentSearchApi` (module-scoped); HTTP `/documents/*` |
| **Dependencies** | `proven-shared`, `proven-core` (files, AuthZ); `SignaturesPort` for ack-sign; `WorkflowPort` for approval/ack campaigns |
| **Events** | `Document*`, `DocumentVersion*`, `DocumentPublished/Withdrawn`, `DocumentAcknowledged`, `AckCampaign*`, `QrSignTarget*` |
| **Database** | Schema `documents` — documents, versions, approvals, acknowledgements, qr_targets, search_projections |
| **Configuration** | Approval policies; retention classes; OCR accept policy |
| **Testing** | Publish immutability; version supersede; ack completion; ACL on restricted docs |
| **Folder Structure** | Standard |
| **Ownership** | `@proven-backend` (document control owners) |

Domain doc: [DOCUMENTS_DOMAIN.md](./DOCUMENTS_DOMAIN.md).

---

### 4.7 `proven-signatures`

| Aspect | Design |
| --- | --- |
| **Purpose** | Proof of assent: packages, slots, capture, guest/magic-link/QR, identity assurance at seal, evidence certificates metadata; **not** subject business meaning. |
| **Public API** | `SignaturePackageApi` (create, seal_slot, void, complete); `MagicLinkApi`; `QrSignSessionApi`; guest HTTP routes (token-scoped); HTTP `/signatures/*` |
| **Dependencies** | `proven-shared`, `proven-core` (AuthZ, files for stroke/image); query ports to Documents (version validation); notifies via events; WorkflowPort for reminders |
| **Events** | `SignaturePackage*`, `SignatureSlot*`, `MagicLink*`, `QrSignSession*`, `EvidenceCertificate*` |
| **Database** | Schema `signatures` — packages, slots, captures, magic_links, qr_sessions, certificates, assurance_records |
| **Configuration** | Link TTL, offline seal policy flags, IDV requirements |
| **Testing** | Sequential/parallel slots; seal immutability; guest scope isolation; void+new package |
| **Folder Structure** | Standard |
| **Ownership** | `@proven-backend` + `@proven-security` (guest token paths) |

Domain doc: [SIGNATURES_DOMAIN.md](./SIGNATURES_DOMAIN.md).

---

### 4.8 `proven-notifications`

| Aspect | Design |
| --- | --- |
| **Purpose** | In-app notifications, preferences, fan-out to channels (email, push, Teams, WhatsApp); delivery attempts orchestration to Go workers; digests; escalation hooks with Workflows. |
| **Public API** | `NotificationApi` (`notify`, list, mark_read); `PreferenceApi`; internal `DeliveryAttemptApi` for workers; HTTP `/notifications/*` |
| **Dependencies** | `proven-shared`, `proven-core` (AuthZ, principal resolution); does **not** own channel provider secrets beyond config refs; WorkflowPort for escalation/digest schedules |
| **Events** | `NotificationCreated/Read`, `DeliveryAttempt*`, `DigestBatch*`, preference changed; consumes many domain events as triggers (via handlers registered in app layer) |
| **Database** | Schema `notifications` — notifications, preferences, delivery_attempts, digest_batches, channel_bindings |
| **Configuration** | Provider enablement, quiet hours, template keys, rate limits |
| **Testing** | Preference gating; idempotent notify; no secrets in payloads; mark-read optimistic-safe |
| **Folder Structure** | Standard |
| **Ownership** | `@proven-backend` (notifications owners) |

Domain doc: [NOTIFICATIONS_DOMAIN.md](./NOTIFICATIONS_DOMAIN.md).

---

### 4.9 `proven-training`

| Aspect | Design |
| --- | --- |
| **Purpose** | Courses, requirements, assignments, completions, renewals, competency gaps, orientation; enforcement queries for “currency.” |
| **Public API** | `CourseApi`; `RequirementApi`; `AssignmentApi`; `CompletionApi`; `CompetencyApi` (`evaluate_gaps`); HTTP `/training/*` |
| **Dependencies** | `proven-shared`, `proven-core`; ports to People (PersonId), Projects (project requirements); WorkflowPort for renewal/expiry; Files for certificates |
| **Events** | `Course*`, `TrainingAssignment*`, `TrainingCompletion*`, `CompetencyGap*`, `Renewal*`, `Orientation*` |
| **Database** | Schema `training` — courses, requirements, assignments, completions, gaps |
| **Configuration** | Renewal windows; orientation course defaults |
| **Testing** | Gap open/close; expiry; assignment upsert idempotency; currency evaluation |
| **Folder Structure** | Standard |
| **Ownership** | `@proven-backend` (training owners) |

Domain doc: [TRAINING_DOMAIN.md](./TRAINING_DOMAIN.md).

---

### 4.10 `proven-cor`

| Aspect | Design |
| --- | --- |
| **Purpose** | COR/SECOR frameworks, mappings, readiness scoring, gaps, evidence packages metadata, audit engagements; extensible packs. |
| **Public API** | `FrameworkApi`; `MappingApi`; `ReadinessApi`; `GapApi`; `EvidencePackageApi`; `EngagementApi`; HTTP `/cor/*` |
| **Dependencies** | `proven-shared`, `proven-core`; **query ports** to Safety/Training/Equipment/Documents/Signatures for provenance refs—never their SQL; WorkflowPort for prep/engagement; Files for package blobs |
| **Events** | `ReadinessRecalculated`, `Gap*`, `EvidencePackage*`, `AuditEngagement*`, `Framework*` |
| **Database** | Schema `cor` — frameworks, elements, mappings, readiness, gaps, packages, engagements, scorecards |
| **Configuration** | Active framework pack; scoring weights |
| **Testing** | Readiness recompute idempotency; package assembly authz; gap SLA fields |
| **Folder Structure** | Standard |
| **Ownership** | `@proven-backend` (COR owners) |

Domain doc: [COR_DOMAIN.md](./COR_DOMAIN.md).

---

### 4.11 `proven-analytics`

| Aspect | Design |
| --- | --- |
| **Purpose** | Metric catalog, dashboard/report definitions, subscriptions, export jobs, **query API** over ClickHouse (read); not OLTP SoR for operational entities. |
| **Public API** | `MetricCatalogApi`; `DashboardApi`; `ReportApi`; `ExportJobApi`; `AnalyticsQueryApi` (AuthZ-scoped); HTTP `/analytics/*`, `/reports` |
| **Dependencies** | `proven-shared`, `proven-core` (AuthZ); ClickHouse client; Files for export artifacts; WorkflowPort for export workflow; **no** writes to domain schemas |
| **Events** | `MetricDefinitionPublished`, `DashboardDefinitionPublished`, `ReportDefinitionPublished`, `ExportJob*`, `AnalyticsSubscription*`, `AnalyticsProjectionRebuilt` |
| **Database** | Schema `analytics` (Postgres) — definitions, schedules, export_jobs, checkpoints metadata; **facts in ClickHouse** (see Data Warehouse doc) |
| **Configuration** | CH DSN, freshness SLO display, export caps, sensitivity classes |
| **Testing** | AuthZ scope injection on queries; export job state machine; catalog permission filter |
| **Folder Structure** | Standard (+ `infrastructure/clickhouse`) |
| **Ownership** | `@proven-backend` (analytics owners) |

Domain docs: [ANALYTICS_DOMAIN.md](./ANALYTICS_DOMAIN.md), [DATA_WAREHOUSE_ARCHITECTURE.md](./DATA_WAREHOUSE_ARCHITECTURE.md).

---

### 4.12 `proven-admin`

| Aspect | Design |
| --- | --- |
| **Purpose** | Administration **facade**: tenant branding, API keys, integration registrations UI/API aggregation, builder drafts/publish orchestration, admin health views—**orchestrates** Core/Projects/Integrations; minimal own SoR. |
| **Public API** | `BrandingApi`; `ApiKeyApi` (delegates secrets hashing to Core/Admin store); `BuilderApi`; `AdminDashboardQuery` (composes module queries); HTTP `/admin/*` |
| **Dependencies** | `proven-shared`, `proven-core` (heavy); ports to Projects, Integrations, Notifications, Analytics; WorkflowPort for onboarding/builder publish/api key expiry |
| **Events** | `BrandingUpdated`, `ApiKeyIssued/Rotated/Revoked`, `BuilderDraft*`, `BuilderPublish*`, `AdminHealthSnapshot*` |
| **Database** | Schema `admin` — branding, api_keys (hashed), builder_drafts, publications, integration_bindings refs |
| **Configuration** | Builder allowed target modules; API key TTL defaults |
| **Testing** | Facade does not bypass AuthZ; publish calls module ports; key plaintext never persisted |
| **Folder Structure** | Standard (+ `application/facades`) |
| **Ownership** | `@proven-backend` (admin owners) |

Domain doc: [ADMINISTRATION_DOMAIN.md](./ADMINISTRATION_DOMAIN.md).

---

### 4.13 `proven-integrations`

| Aspect | Design |
| --- | --- |
| **Purpose** | External system connectors: webhook endpoints (inbound), outbound webhook subscriptions, ERP/HRIS/SSO auxiliary adapters **configuration**, sync job metadata, health probes—**not** a dumping ground for domain rules. Transformations call owning module public APIs. |
| **Public API** | `IntegrationRegistrationApi`; `WebhookSubscriptionApi`; `InboundWebhookHandler` (verify signature → map → module command); `ConnectorHealthApi`; HTTP `/integrations/*`, `/webhooks/inbound/{connector}` |
| **Dependencies** | `proven-shared`, `proven-core` (AuthZ, API keys/OIDC client registry as applicable); ports to People/Projects/Training/etc. for upsert commands; WorkflowPort for health poll / sync; **no** direct writes to foreign schemas |
| **Events** | `IntegrationRegistered/Updated/Disabled`, `WebhookDelivery*`, `ConnectorHealthChanged`, `SyncJob*` |
| **Database** | Schema `integrations` — connectors, subscriptions, sync_jobs, health_snapshots, inbound_receipts (idempotency) |
| **Configuration** | Per-connector secrets refs (vault), egress allowlists, retry policies |
| **Testing** | Signature verification; idempotent inbound; egress allowlist; mapping failures dead-letter |
| **Folder Structure** | Standard (+ `connectors/<name>/` adapter modules) |
| **Ownership** | `@proven-backend` + `@proven-sre` (egress/health) |

---

## 5. Cross-Crate Dependency Matrix (Allowed)

| Crate | May depend on (public APIs) |
| --- | --- |
| `proven-core` | `proven-shared` only |
| `proven-projects` | core |
| `proven-people` | core |
| `proven-safety` | core + signatures/documents/projects/workflows **ports** |
| `proven-equipment` | core + projects/workflows ports |
| `proven-documents` | core + signatures/workflows ports |
| `proven-signatures` | core + documents (query) + workflows ports |
| `proven-notifications` | core + workflows ports |
| `proven-training` | core + people/projects/workflows ports |
| `proven-cor` | core + read ports to safety/training/equipment/documents/signatures + workflows |
| `proven-analytics` | core + workflows (+ CH) |
| `proven-admin` | core + projects/integrations/analytics/notifications/workflows ports |
| `proven-integrations` | core + target module command ports + workflows |
| `proven-workflows` | core (+ used by all) |
| `proven-platform` | all modules (composition) |

Ports are traits defined by the **provider** crate (or a thin `ports` module) and implemented/adapted at platform wiring—never by importing provider `infrastructure`.

---

## 6. Events Convention (All Modules)

- Past-tense names; envelope per [EVENT_CATALOG.md](./EVENT_CATALOG.md).  
- Written in **same transaction** as state change via outbox table (platform publisher).  
- No passwords, magic secrets, stroke bitmaps, medical bodies.  
- Consumers: Notifications, Analytics workers, Search indexer, other modules’ application event handlers.

---

## 7. Database Convention (All Modules)

- One PostgreSQL **schema per module** (`core`, `projects`, `people`, …).  
- RLS via `app.tenant_id`.  
- Soft delete + audit where required.  
- **No cross-schema FKs** for coupling; store foreign UUIDs only.  
- Migrations live under `db/migrations` owned by the module team.

---

## 8. Configuration Convention

- Module section in root config / env: `PROVEN_<MODULE>_…` or TOML `[modules.safety]`.  
- Secrets only via platform secret injection—never in crate defaults.  
- Feature flags via Core `FeatureFlagApi` where tenant-toggleable.

---

## 9. Testing Convention

| Layer | Expectation |
| --- | --- |
| Domain unit | Aggregates/invariants pure |
| Application | Use cases with fake ports |
| Infrastructure integ | SQLx + test DB via `proven-test-support` |
| HTTP | Axum router tests with AuthZ fixtures |
| Property/idempotency | Offline mutation ids, webhook receipts |

CI: `cargo test -p proven-<module>` path-filtered.

---

## 10. Ownership Summary

| Crate | Primary owners |
| --- | --- |
| `proven-shared` / `proven-platform` / `proven-test-support` / `proven-workflows` | Platform |
| `proven-core` | Backend + Security |
| `proven-projects` / `proven-people` / `proven-documents` / `proven-training` / `proven-notifications` / `proven-analytics` / `proven-admin` / `proven-cor` | Backend (domain specialists) |
| `proven-safety` | Safety + Backend |
| `proven-equipment` | Equipment + Backend |
| `proven-signatures` | Backend + Security |
| `proven-integrations` | Backend + SRE |
| `proven-contracts` | Backend + Frontend (contracts) |

Align GitHub CODEOWNERS paths under `/crates/modules/proven-*/`.

---

## 11. Binary Host

| Aspect | Design |
| --- | --- |
| **Crate** | `apps/api` (`proven-api`) |
| **Purpose** | `main`: load config, build platform app, serve |
| **Dependencies** | `proven-platform` only (ideally) |
| **Testing** | Smoke / health e2e |

---

## 12. Success Criteria

1. Each bounded context is one crate with a documented public API.  
2. No crate reaches into another’s schema or domain internals.  
3. Core AuthZ is the sole permission authority.  
4. Events + Temporal ports integrate modules without shared transactions across schemas.  
5. Integrations and Admin remain facades/adapters—not second domain models.  
6. Folder layout and testing strategy are uniform for onboarding velocity.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Lead Rust Engineering | Complete crate catalog |

---

*End of Rust Crate Catalog*
