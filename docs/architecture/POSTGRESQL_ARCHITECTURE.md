# Proven — PostgreSQL Enterprise Database Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | PostgreSQL Database Architecture |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Database Architecture / Platform |
| **Audience** | Engineering, SRE, Security, Compliance |
| **Last updated** | 2026-08-03 |
| **Companion docs** | Domain module architectures under `docs/architecture/*_DOMAIN.md`, [System Architecture](./SYSTEM_ARCHITECTURE.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document defines the **complete PostgreSQL architecture** for Proven as the operational system of record.

It covers multi-tenancy, soft deletes, audit logging, version history, JSONB usage, indexing, constraints, foreign keys, partitioning, row-level security (RLS), backup, migrations, PostGIS readiness, and future replication—plus a **table catalog** for Core, Projects, People, Safety, Equipment, Documents, Training, COR, Notifications, Analytics metadata, Digital Signatures, and Workflow tracking.

**No SQL DDL is included.** This is enterprise database design documentation.

**Out of scope as Postgres SoR:** ClickHouse analytical facts, Redis cache, R2 object bytes, Temporal workflow runtime state (only correlation/tracking rows in Postgres).

---

## 2. Architectural Principles

1. **One primary PostgreSQL cluster** initially (HA primary + replicas later).  
2. **Schema-per-module** ownership aligned to bounded contexts.  
3. **No cross-schema foreign keys** — modules reference foreign ids as UUIDs only.  
4. **Tenant id on every tenant-owned row** — defense in depth with RLS.  
5. **Soft delete by default** for business entities; hard delete only via retention jobs.  
6. **Append-only audit** in `core` — never update/delete audit facts.  
7. **Expand/contract migrations** — no long locks; backward-compatible deploys.  
8. **JSONB for extensibility, not for relational truth** that needs heavy constraint/join.  
9. **Partition hot/growing tables** by time (and sometimes tenant) before pain.  
10. **Postgres is OLTP** — heavy trends go to ClickHouse.

---

## 3. Logical Database Layout

### 3.1 Databases

| Database | Purpose |
| --- | --- |
| `proven` | Primary OLTP application database |
| `proven_analytics` *(optional later)* | Only if analytics metadata must isolate; default keep metadata in `proven.analytics` |

### 3.2 Schemas

| Schema | Module |
| --- | --- |
| `platform` | Outbox, migration ledger aids, job locks |
| `core` | Tenancy, identity, authz, files, audit, flags, license, membership, teams, settings |
| `admin` | Branding, API keys, integrations, builder drafts, health snapshots |
| `projects` | Projects place domain |
| `people` | People / workforce domain |
| `safety` | Safety operations |
| `equipment` | Equipment compliance |
| `documents` | Document control |
| `training` | Training & competency |
| `cor` | COR audit readiness |
| `signatures` | Digital signatures |
| `notifications` | Notifications |
| `analytics` | Analytics metadata (not CH facts) |
| `workflows` | Workflow definition/instance tracking |

Application DB roles: `proven_app` (RLS-forced), `proven_migrator` (DDL), `proven_readonly` (support), `proven_admin` (break-glass).

---

## 4. Cross-Cutting Standards

### 4.1 Multi-Tenancy

| Pattern | Choice |
| --- | --- |
| Model | **Shared database, shared schemas, tenant discriminator column** |
| Discriminator | `tenant_id UUID NOT NULL` on all tenant-owned tables |
| Platform rows | Explicit `tenant_id` null only for true global catalog (rare); prefer system tenant |
| Isolation | RLS + app mandatory predicate + no cross-tenant unique violations |
| Partner companies | Multiple `core.companies` per tenant; visibility via membership/grants |

**Composite uniqueness** always includes `tenant_id` (e.g., project code unique per tenant).

### 4.2 Soft Deletes

| Column | Meaning |
| --- | --- |
| `deleted_at TIMESTAMPTZ NULL` | Soft-deleted when non-null |
| `deleted_by UUID NULL` | Actor principal/user |

Rules:

- Unique indexes are **partial** (`WHERE deleted_at IS NULL`) where business keys must reuse after delete.  
- Default queries filter `deleted_at IS NULL`.  
- Soft delete does not cascade across modules; emit events for consumers.  
- Retention jobs hard-delete or archive after policy.

**Exceptions (no soft delete):** append-only tables (`audit_entries`, outbox completed rows strategy, delivery attempts)—use partition drop / archival instead.

### 4.3 Audit Logging

| Store | Purpose |
| --- | --- |
| `core.audit_entries` | Platform security/compliance audit (API `AuditApi`) |
| Module history tables | Operational timelines (optional), not security substitutes |

Every audit row: actor, action, resource type/id, tenant, correlation/causation, payload digest, occurred_at. **Insert-only.**

### 4.4 Version History

Patterns used across modules:

| Pattern | Use |
| --- | --- |
| **Immutable child versions** | `documents.document_versions` — new row per version |
| **History table** | `*_revisions` storing full row snapshot / JSON patch on change |
| **Event-sourced projection** | Prefer domain events for rebuild; history tables for user-facing lineage |
| **Optimistic locking** | `version INT/BIGINT` (row version) on mutable aggregates |

Published content (document versions, evidence certificates, sealed signature packages) is **immutable**—corrections create new rows or void+replace.

### 4.5 JSONB Usage

**Allowed**

- Form/activity response payloads  
- Builder draft documents  
- Settings values  
- Class-specific equipment profiles  
- Template variable maps  
- Provider raw response snippets (truncated, redacted)  
- Metric widget configs  

**Disallowed as sole storage for**

- Tenant id, status enums used in filters (use columns)  
- Foreign ids that need joins/indexes (use UUID columns)  
- Authorization grants  

**Indexing:** GIN on JSONB only where proven query paths need it; prefer generated columns for hot keys.

### 4.6 Indexes (General)

On nearly all tenant tables:

- `(tenant_id)` or leading `tenant_id` in composites  
- `(tenant_id, id)` PK strategy: prefer `id UUID` PK + separate tenant indexes (or composite PK `(tenant_id, id)`—choose one standard: **`id UUID` global PK + tenant_id column + RLS**)  
- Soft-delete partial uniques  
- `created_at` / `occurred_at` for time lists  
- FK-like reference columns (`project_id`, `person_id`, …) with tenant-leading composites  

Hot paths: status+due dates, membership lookups, unread notifications, readiness, assignment queues.

### 4.7 Constraints

- PKs on all tables  
- `NOT NULL` on tenant_id, type enums, required FKs within schema  
- `CHECK` for status enums, non-empty strings where needed, date ranges (`valid_from <= valid_to`)  
- Partial `UNIQUE` for business keys  
- Exclude constraints rarely (calendar overlaps) where justified  

### 4.8 Foreign Keys

| Scope | Policy |
| --- | --- |
| **Within schema** | Real PostgreSQL FKs (cascade rules explicit; prefer Restrict on soft-delete models) |
| **Across schemas/modules** | **No FK** — store UUID + enforce in application/domain |
| **File objects** | UUID ref to `core.file_objects` without cross-schema FK (or optional FK only if same-DB and team accepts coupling—**default no**) |

### 4.9 Partitioning Strategy

| Table class | Partition key | Strategy |
| --- | --- | --- |
| Audit entries | `occurred_at` | Monthly RANGE |
| Outbox / completed bus | `created_at` | Monthly |
| Notifications + delivery attempts | `created_at` | Monthly |
| Signature captures / certificates meta | `created_at` | Monthly/quarterly |
| Safety activities (high volume) | `created_at` or `closed_at` | Monthly after threshold |
| Attendance / punches | `work_date` | Monthly |
| Workflow instance history | `started_at` | Monthly |
| Analytics export jobs | `created_at` | Quarterly |

Start **unpartitioned**; convert when row counts / bloat demand (e.g., >50–100M rows or heavy time deletes). Use declarative partitioning; attach future partitions via migration automation.

**Tenant-based partitioning** deferred until mega-tenant skew requires it (list/hash by tenant_id on specific tables).

### 4.10 Row Level Security (RLS)

1. Enable RLS on all tenant-owned tables.  
2. Force RLS for `proven_app` role.  
3. Policy pattern: `tenant_id = current_setting('app.tenant_id')::uuid`.  
4. Session GUC set by API middleware per request (transaction-local).  
5. Service/jobs set tenant context per batch item; never run app role with RLS bypass.  
6. Migrator / break-glass roles bypass intentionally.  
7. Platform-wide admin reads use explicit elevated role + audited sessions—not disabled RLS globally.

Optional second policy: project-scope session settings for extra defense on sensitive tables (medical)—prefer app AuthZ primary.

### 4.11 Backup Strategy

| Tier | Method | RPO / RTO guidance |
| --- | --- | --- |
| Continuous | Provider PITR / WAL archiving | RPO minutes |
| Daily | Full base backup | Retention 30–90 days |
| Weekly | Longer retention copy | 6–12 months (policy) |
| Pre-migrate | Snapshot | Until migrate verified |

Rules:

- Encrypted backups  
- Quarterly restore drills  
- Document partition detach ≠ backup  
- Secrets not in DB; backup still treated confidential (PII/PHI)  
- ClickHouse backed up separately  

### 4.12 Migration Strategy

1. Tooling: forward-only migrations per schema ownership (sqitch/goose/atlas/sqlx—implementation choice later).  
2. **Expand/contract**: add columns nullable → dual-write → backfill → switch → drop obsolete.  
3. Lock budget: avoid full table rewrites on hot tables; use batched backfills.  
4. RLS/policy changes in migrations reviewed by security.  
5. Partition creation automated ahead of time.  
6. Migration runs in CI dry-run + staging; prod gated in deploy.  
7. Cross-schema migrations forbidden in one module PR without platform review.  
8. Data backfills as versioned jobs, not unbounded migrate transactions.

### 4.13 PostGIS Readiness

| Item | Design |
| --- | --- |
| Extension | Prepare for `postgis` on primary when location features require it |
| Initial storage | `site` lat/long as `DOUBLE PRECISION` or `GEOGRAPHY` columns nullable without forcing extension day one |
| Tables likely to gain geography | `projects.project_locations`, `projects.project_areas`, `equipment.assets` (optional site point), `safety.observations` |
| Indexes | GiST on geography when enabled |
| RLS | Unchanged |
| Migration path | Additive columns → backfill → enable extension in controlled change window |

Do not block OLTP on GIS features until product requires map queries.

### 4.14 Future Replication

| Phase | Topology |
| --- | --- |
| Now | Single primary |
| Next | Async read replicas for heavy read APIs / admin audit search |
| Later | Regional read replicas; primary writes pinned |
| Extreme | Logical replication of selected schemas to analytics staging (prefer events→CH) |

Rules: app uses primary for writes; replicas for designated read roles; lag monitoring; RLS settings must flow to replicas.

---

## 5. Common Column Sets

**Identity:** `id UUID PK`, `tenant_id UUID NOT NULL`, `created_at`, `created_by`, `updated_at`, `updated_by`, `deleted_at`, `deleted_by`, `row_version BIGINT`

**Reference-only foreign ids (no FK):** `project_id`, `person_id`, `company_id`, `user_id`, `file_object_id`, …

---

## 6. Platform Schema Tables

### 6.1 `platform.outbox_messages`

| Aspect | Design |
| --- | --- |
| **Purpose** | Transactional outbox for NATS/domain events |
| **Relationships** | None cross-module; payload references resource ids |
| **Indexes** | `(status, available_at)`; `(created_at)`; unique `(id)`; partial unpublished |
| **Constraints** | Status check; NOT NULL tenant_id, event_type, payload |
| **Retention** | Soft archive/hard delete after publish+N days (e.g., 7–30); partition drop |
| **Expected growth** | Very High — partition monthly |

### 6.2 `platform.idempotency_keys`

| Aspect | Design |
| --- | --- |
| **Purpose** | Deduplicate client/offline mutations |
| **Relationships** | Logical to principal/tenant |
| **Indexes** | Unique `(tenant_id, actor_id, mutation_key)` |
| **Constraints** | Expiry NOT NULL |
| **Retention** | Hard delete after TTL (e.g., 7–30 days) |
| **Expected growth** | High |

### 6.3 `platform.schema_migrations` *(tool-managed)*

| Aspect | Design |
| --- | --- |
| **Purpose** | Migration version ledger |
| **Relationships** | n/a |
| **Indexes** | Unique version |
| **Constraints** | PK version |
| **Retention** | Forever |
| **Expected growth** | Low |

---

## 7. Core Schema Tables

### 7.1 `core.tenants`

| Aspect | Design |
| --- | --- |
| **Purpose** | Customer workspace root |
| **Relationships** | Parent of nearly all tenant data (logical) |
| **Indexes** | PK `id`; unique `slug` |
| **Constraints** | Status check; region NOT NULL |
| **Retention** | Soft delete; hard delete only after legal exit |
| **Expected growth** | Low (thousands) |

### 7.2 `core.companies`

| Aspect | Design |
| --- | --- |
| **Purpose** | Legal/operating companies in a tenant |
| **Relationships** | FK → tenants; referenced by projects participants, people employments (logical) |
| **Indexes** | `(tenant_id)`; unique `(tenant_id, legal_name)` partial; `(tenant_id, company_type)` |
| **Constraints** | Type check; tenant FK |
| **Retention** | Soft delete; long retain |
| **Expected growth** | Low–Medium |

### 7.3 `core.org_units`

| Aspect | Design |
| --- | --- |
| **Purpose** | Org hierarchy nodes |
| **Relationships** | FK tenant; self-FK parent_org_unit_id |
| **Indexes** | `(tenant_id, parent_id)`; unique `(tenant_id, code)` partial |
| **Constraints** | No cyclic parent (app enforced) |
| **Retention** | Soft delete |
| **Expected growth** | Low–Medium |

### 7.4 `core.users`

| Aspect | Design |
| --- | --- |
| **Purpose** | Login identity |
| **Relationships** | FK tenant; logical `person_id` |
| **Indexes** | Unique `(tenant_id, email)` partial; `(tenant_id, person_id)` |
| **Constraints** | Status check; email format check optional |
| **Retention** | Soft delete; credentials purged per policy |
| **Expected growth** | Medium |

### 7.5 `core.credentials`

| Aspect | Design |
| --- | --- |
| **Purpose** | Password/webauthn secrets |
| **Relationships** | FK → users |
| **Indexes** | Unique `user_id` per type |
| **Constraints** | NOT NULL hash/secret material columns |
| **Retention** | Hard delete with user purge; never soft-retain hashes longer than needed |
| **Expected growth** | Medium |

### 7.6 `core.external_identity_links`

| Aspect | Design |
| --- | --- |
| **Purpose** | SSO subject mapping |
| **Relationships** | FK → users |
| **Indexes** | Unique `(provider, subject)`; `(user_id)` |
| **Constraints** | Provider NOT NULL |
| **Retention** | Soft/hard with user |
| **Expected growth** | Medium |

### 7.7 `core.sessions`

| Aspect | Design |
| --- | --- |
| **Purpose** | Revocable sessions |
| **Relationships** | FK → users |
| **Indexes** | `(user_id)`; `(expires_at)`; token hash unique |
| **Constraints** | Expiry NOT NULL |
| **Retention** | Hard delete expired (days–weeks) |
| **Expected growth** | High (partition optional) |

### 7.8 `core.permissions`

| Aspect | Design |
| --- | --- |
| **Purpose** | Global permission catalog codes |
| **Relationships** | Referenced by role_permissions |
| **Indexes** | Unique `code` |
| **Constraints** | Code format |
| **Retention** | Forever (retire flag) |
| **Expected growth** | Low |

### 7.9 `core.roles`

| Aspect | Design |
| --- | --- |
| **Purpose** | Role definitions (system + tenant custom) |
| **Relationships** | FK tenant (nullable for system); role_permissions |
| **Indexes** | Unique `(tenant_id, name)` partial |
| **Constraints** | Kind check |
| **Retention** | Soft delete / retire |
| **Expected growth** | Low–Medium |

### 7.10 `core.role_permissions`

| Aspect | Design |
| --- | --- |
| **Purpose** | Role↔permission bindings |
| **Relationships** | FK roles, permissions |
| **Indexes** | Unique `(role_id, permission_code)` |
| **Constraints** | FKs |
| **Retention** | Hard with role updates |
| **Expected growth** | Low–Medium |

### 7.11 `core.access_grants`

| Aspect | Design |
| --- | --- |
| **Purpose** | Principal role grants in scope |
| **Relationships** | FK users/roles; logical project/org ids |
| **Indexes** | `(tenant_id, user_id)`; `(tenant_id, scope_type, scope_id)`; unique active grant tuple partial |
| **Constraints** | Scope type check |
| **Retention** | Soft revoke (`revoked_at`) retain history |
| **Expected growth** | Medium–High |

### 7.12 `core.project_memberships`

| Aspect | Design |
| --- | --- |
| **Purpose** | Authoritative person/user↔project participation |
| **Relationships** | Logical project_id, person_id, user_id; FK tenant |
| **Indexes** | Unique `(tenant_id, project_id, person_id)` active partial; `(tenant_id, person_id)`; `(tenant_id, project_id, status)` |
| **Constraints** | Status check |
| **Retention** | Soft end; retain for history |
| **Expected growth** | High |

### 7.13 `core.teams` / `core.team_members`

| Aspect | Design |
| --- | --- |
| **Purpose** | Teams and membership |
| **Relationships** | FK team→tenant; members→team; logical project_id |
| **Indexes** | Unique `(tenant_id, project_id, name)` partial; `(team_id, person_id)` unique active |
| **Constraints** | FKs within core |
| **Retention** | Soft delete |
| **Expected growth** | Medium |

### 7.14 `core.file_objects`

| Aspect | Design |
| --- | --- |
| **Purpose** | Object storage metadata |
| **Relationships** | Logical owner module/resource |
| **Indexes** | `(tenant_id, status)`; `(storage_key)` unique; `(checksum)` |
| **Constraints** | Status/size checks |
| **Retention** | Soft delete + R2 lifecycle; hard purge per retention class |
| **Expected growth** | Very High |

### 7.15 `core.audit_entries`

| Aspect | Design |
| --- | --- |
| **Purpose** | Append-only platform audit |
| **Relationships** | Logical actor/resource |
| **Indexes** | `(tenant_id, occurred_at DESC)`; `(tenant_id, resource_type, resource_id)`; `(correlation_id)` |
| **Constraints** | Insert-only privileges; NOT NULL action/actor/occurred_at |
| **Retention** | Long (years); partition drop per legal policy |
| **Expected growth** | Very High — **partition monthly** |

### 7.16 `core.settings_bundles` / `core.setting_entries`

| Aspect | Design |
| --- | --- |
| **Purpose** | Scoped settings |
| **Relationships** | Bundle per scope; entries FK bundle |
| **Indexes** | Unique `(tenant_id, scope_type, scope_id, key)` |
| **Constraints** | JSONB value schema validated in app |
| **Retention** | Soft overwrite history optional via revisions |
| **Expected growth** | Medium |

### 7.17 `core.feature_flags` / `core.feature_flag_overrides`

| Aspect | Design |
| --- | --- |
| **Purpose** | Flags and targeting overrides |
| **Relationships** | Override FK flag |
| **Indexes** | Unique flag key; overrides `(flag_id, tenant_id, actor_id)` |
| **Constraints** | Key unique |
| **Retention** | Soft retire flags |
| **Expected growth** | Low |

### 7.18 `core.licenses` / `core.seat_allocations` / `core.module_entitlements`

| Aspect | Design |
| --- | --- |
| **Purpose** | Commercial entitlements & seats |
| **Relationships** | FK tenant; child allocations |
| **Indexes** | `(tenant_id)` unique active license partial; seat type indexes |
| **Constraints** | Status/period checks |
| **Retention** | Keep historical licenses |
| **Expected growth** | Low |

---

## 8. Admin Schema Tables

### 8.1 `admin.tenant_branding`

| Aspect | Design |
| --- | --- |
| **Purpose** | Tenant brand tokens/assets |
| **Relationships** | Logical file_object ids; 1:1 tenant |
| **Indexes** | Unique `tenant_id` |
| **Constraints** | — |
| **Retention** | Soft update in place + optional revision row |
| **Expected growth** | Low |

### 8.2 `admin.api_clients` / `admin.api_keys`

| Aspect | Design |
| --- | --- |
| **Purpose** | Machine clients and hashed API keys |
| **Relationships** | Keys FK clients; logical Core principal link |
| **Indexes** | Unique key prefix; `(tenant_id, status)` |
| **Constraints** | Hash NOT NULL; never store raw key |
| **Retention** | Revoked keys retained limited period then hard delete |
| **Expected growth** | Low–Medium |

### 8.3 `admin.integrations`

| Aspect | Design |
| --- | --- |
| **Purpose** | Integration registry metadata |
| **Relationships** | Optional api_client_id FK |
| **Indexes** | `(tenant_id, type, status)` |
| **Constraints** | Status check; secret_ref not plaintext |
| **Retention** | Soft delete |
| **Expected growth** | Low |

### 8.4 `admin.builder_drafts` / `admin.builder_publications`

| Aspect | Design |
| --- | --- |
| **Purpose** | Builder studio drafts & publish records |
| **Relationships** | Publication → draft; JSONB body |
| **Indexes** | `(tenant_id, kind, status)`; `(draft_id)` |
| **Constraints** | Kind/status checks |
| **Retention** | Discarded drafts purged 90 days; publications long |
| **Expected growth** | Medium |

### 8.5 `admin.admin_dashboard_definitions` / `admin.system_health_snapshots`

| Aspect | Design |
| --- | --- |
| **Purpose** | Admin home config; health snapshots |
| **Relationships** | — |
| **Indexes** | Snapshots `(captured_at DESC)` |
| **Constraints** | — |
| **Retention** | Snapshots 30–90 days |
| **Expected growth** | Medium (snapshots) |

### 8.6 `admin.billing_accounts` *(future)*

| Aspect | Design |
| --- | --- |
| **Purpose** | Billing stub |
| **Relationships** | 1:1 tenant |
| **Indexes** | Unique tenant_id |
| **Constraints** | — |
| **Retention** | Forever commercial |
| **Expected growth** | Low |

---

## 9. Projects Schema Tables

### 9.1 `projects.projects`

| Aspect | Design |
| --- | --- |
| **Purpose** | Construction Place aggregate |
| **Relationships** | Logical tenant; children participants/areas |
| **Indexes** | Unique `(tenant_id, code)` partial; `(tenant_id, status)`; `(tenant_id, updated_at DESC)` |
| **Constraints** | Status check; row_version |
| **Retention** | Soft archive; long retain closed |
| **Expected growth** | Medium |

### 9.2 `projects.project_participants`

| Aspect | Design |
| --- | --- |
| **Purpose** | Prime/Sub/Client company engagement |
| **Relationships** | FK project; logical company_id |
| **Indexes** | Unique `(project_id, company_id, participation_role)` active partial; `(tenant_id, company_id)` |
| **Constraints** | Role/status checks; app: one active prime |
| **Retention** | Soft remove |
| **Expected growth** | Medium |

### 9.3 `projects.project_locations` / `projects.project_areas`

| Aspect | Design |
| --- | --- |
| **Purpose** | Site location & areas (PostGIS-ready columns) |
| **Relationships** | FK project; area FK project |
| **Indexes** | Unique `(project_id, code)` areas; geo index later |
| **Constraints** | — |
| **Retention** | Soft deactivate |
| **Expected growth** | Medium |

### 9.4 `projects.project_settings`

| Aspect | Design |
| --- | --- |
| **Purpose** | Project-scoped settings |
| **Relationships** | Unique FK/ logical 1:1 project |
| **Indexes** | Unique `project_id` |
| **Constraints** | JSONB settings |
| **Retention** | With project |
| **Expected growth** | Medium |

### 9.5 `projects.required_controls` / `projects.form_bindings` / `projects.document_links` / `projects.equipment_requirements` / `projects.team_links`

| Aspect | Design |
| --- | --- |
| **Purpose** | Requirement & link entities |
| **Relationships** | FK project; logical foreign module ids |
| **Indexes** | `(project_id, type)`; unique link tuples partial |
| **Constraints** | Type checks |
| **Retention** | Soft remove; keep for history if referenced |
| **Expected growth** | Medium–High |

### 9.6 `projects.project_templates` (+ template child tables)

| Aspect | Design |
| --- | --- |
| **Purpose** | Reusable project blueprints |
| **Relationships** | Tenant-owned; child slots/controls |
| **Indexes** | `(tenant_id, status)` |
| **Constraints** | — |
| **Retention** | Soft retire |
| **Expected growth** | Low–Medium |

### 9.7 `projects.dashboard_projections`

| Aspect | Design |
| --- | --- |
| **Purpose** | Place dashboard counters |
| **Relationships** | 1:1 project logical |
| **Indexes** | Unique `project_id` |
| **Constraints** | Rebuildable |
| **Retention** | Rebuild anytime |
| **Expected growth** | Medium |

---

## 10. People Schema Tables

### 10.1 `people.persons`

| Aspect | Design |
| --- | --- |
| **Purpose** | Human profile SoR |
| **Relationships** | Logical user_id, company links via employment |
| **Indexes** | `(tenant_id, status)`; name search `(tenant_id, last_name, first_name)`; unique external_ref partial |
| **Constraints** | Status check |
| **Retention** | Soft deactivate; long retain for evidence refs |
| **Expected growth** | High |

### 10.2 `people.workforce_role_assignments` / `people.trade_assignments`

| Aspect | Design |
| --- | --- |
| **Purpose** | Worker/Supervisor/Manager tags; trades |
| **Relationships** | FK person |
| **Indexes** | `(person_id)`; `(tenant_id, trade_code)` |
| **Constraints** | Role/trade checks |
| **Retention** | Soft end-date |
| **Expected growth** | High |

### 10.3 `people.emergency_contacts`

| Aspect | Design |
| --- | --- |
| **Purpose** | Emergency contacts (PII) |
| **Relationships** | FK person |
| **Indexes** | `(person_id)` |
| **Constraints** | — |
| **Retention** | Strict; purge with person policy |
| **Expected growth** | High |

### 10.4 `people.medical_restrictions`

| Aspect | Design |
| --- | --- |
| **Purpose** | Fit-for-work restrictions (PHI-sensitive) |
| **Relationships** | FK person |
| **Indexes** | `(person_id, status)`; `(tenant_id, fit_signal)` |
| **Constraints** | Severity/period checks |
| **Retention** | Strict legal retention; separate access |
| **Expected growth** | Medium |

### 10.5 `people.employments` / `people.contractor_engagements`

| Aspect | Design |
| --- | --- |
| **Purpose** | Company relationships |
| **Relationships** | FK person; logical company_id |
| **Indexes** | `(person_id, status)`; `(tenant_id, company_id)` |
| **Constraints** | Period checks |
| **Retention** | Soft end; retain history |
| **Expected growth** | High |

### 10.6 `people.availability_calendars` / `people.availability_windows` / `people.availability_exceptions`

| Aspect | Design |
| --- | --- |
| **Purpose** | Availability |
| **Relationships** | FK person |
| **Indexes** | `(person_id, range)` |
| **Constraints** | Range checks |
| **Retention** | 1–2 years typical |
| **Expected growth** | High |

### 10.7 `people.attendance_records`

| Aspect | Design |
| --- | --- |
| **Purpose** | Workforce attendance |
| **Relationships** | FK person; logical project_id |
| **Indexes** | `(tenant_id, work_date)`; `(person_id, work_date)` unique partial |
| **Constraints** | Status check |
| **Retention** | 2–7 years per policy; **partition by work_date** |
| **Expected growth** | Very High |

### 10.8 `people.certification_profile_entries`

| Aspect | Design |
| --- | --- |
| **Purpose** | Profile cards referencing Training/Documents |
| **Relationships** | FK person; logical completion/document ids |
| **Indexes** | `(person_id)` |
| **Constraints** | — |
| **Retention** | Soft remove |
| **Expected growth** | High |

### 10.9 Projection tables

`people.competency_profile_projections`, `people.assignment_views`, `people.signature_history_items`, `people.person_history_entries`

| Aspect | Design |
| --- | --- |
| **Purpose** | Rebuildable profile projections |
| **Relationships** | FK/logical person |
| **Indexes** | By person_id; history by occurred_at |
| **Constraints** | Rebuildable |
| **Retention** | Rebuild; history entries retain years |
| **Expected growth** | High–Very High |

---

## 11. Safety Schema Tables

### 11.1 `safety.activity_type_definitions`

| Aspect | Design |
| --- | --- |
| **Purpose** | FLHA/toolbox/inspection type catalog |
| **Relationships** | Tenant-owned |
| **Indexes** | Unique `(tenant_id, code)` partial |
| **Constraints** | Schema JSONB optional |
| **Retention** | Soft retire |
| **Expected growth** | Low–Medium |

### 11.2 `safety.safety_activities`

| Aspect | Design |
| --- | --- |
| **Purpose** | Activity instances |
| **Relationships** | FK type; logical project/person; children entries |
| **Indexes** | `(tenant_id, project_id, status)`; `(tenant_id, created_at DESC)`; `(tenant_id, activity_type_id, status)` |
| **Constraints** | Status check; row_version |
| **Retention** | Long (COR); soft void; **partition by created_at** when large |
| **Expected growth** | Very High |

### 11.3 Child activity tables

`safety.activity_participants`, `attendance_entries`, `hazard_entries`, `control_entries`, `risk_assessment_entries`, `response_entries`, `attachment_refs`, `weather_snapshots`, `procedure_ack_refs`, `signature_package_refs`

| Aspect | Design |
| --- | --- |
| **Purpose** | Activity structure |
| **Relationships** | FK → safety_activities |
| **Indexes** | `(activity_id)`; hazard library refs |
| **Constraints** | FKs within safety |
| **Retention** | With activity |
| **Expected growth** | Very High |

### 11.4 `safety.hazard_library_items` / `safety.control_library_items` / `safety.hazard_control_suggestions`

| Aspect | Design |
| --- | --- |
| **Purpose** | Reusable libraries |
| **Relationships** | Suggestion links hazard↔control |
| **Indexes** | Unique codes per tenant; tags GIN optional |
| **Constraints** | Status |
| **Retention** | Soft retire (keep snapshots on activities) |
| **Expected growth** | Medium |

### 11.5 `safety.risk_matrix_definitions` (+ levels/cells)

| Aspect | Design |
| --- | --- |
| **Purpose** | Risk matrix packs |
| **Relationships** | Child levels/cells |
| **Indexes** | `(tenant_id, status)` |
| **Constraints** | — |
| **Retention** | Version/retire |
| **Expected growth** | Low |

### 11.6 `safety.corrective_actions` (+ updates/attachments)

| Aspect | Design |
| --- | --- |
| **Purpose** | CA aggregate |
| **Relationships** | Logical source activity; owner person |
| **Indexes** | `(tenant_id, status, due_at)`; `(project_id, status)` |
| **Constraints** | Status/due checks |
| **Retention** | Long |
| **Expected growth** | High |

### 11.7 `safety.incident_cases` (+ investigation children)

| Aspect | Design |
| --- | --- |
| **Purpose** | Incidents / serious near misses |
| **Relationships** | Linked activities/CAs logical |
| **Indexes** | `(tenant_id, status)`; `(project_id)` |
| **Constraints** | — |
| **Retention** | Long legal |
| **Expected growth** | Medium |

### 11.8 `safety.safety_bulletins` (+ audience/acks)

| Aspect | Design |
| --- | --- |
| **Purpose** | Bulletins |
| **Relationships** | Ack persons; optional doc refs |
| **Indexes** | `(tenant_id, status)`; ack `(bulletin_id, person_id)` unique |
| **Constraints** | — |
| **Retention** | Long |
| **Expected growth** | Medium |

### 11.9 `safety.permit_cases` / `safety.lift_plan_cases` (+ children)

| Aspect | Design |
| --- | --- |
| **Purpose** | Permits & lift plans |
| **Relationships** | Logical asset/project; signature refs |
| **Indexes** | `(tenant_id, project_id, status)` |
| **Constraints** | Status |
| **Retention** | Long |
| **Expected growth** | Medium–High |

### 11.10 `safety.daily_logs` / `safety.daily_log_entries`

| Aspect | Design |
| --- | --- |
| **Purpose** | Daily logs |
| **Relationships** | Unique open log per project/date/shift |
| **Indexes** | Unique `(project_id, work_date, shift)` partial open/closed policy |
| **Constraints** | — |
| **Retention** | Years |
| **Expected growth** | High — partition by work_date optional |

### 11.11 `safety.procedure_bindings` / `safety.reporting_projections`

| Aspect | Design |
| --- | --- |
| **Purpose** | SWP/SJP bindings; report projections |
| **Relationships** | Logical document_version_id |
| **Indexes** | By project |
| **Constraints** | — |
| **Retention** | Soft; projections rebuildable |
| **Expected growth** | Medium |

---

## 12. Equipment Schema Tables

### 12.1 `equipment.asset_type_definitions` / `equipment.inspection_checklist_definitions`

| Aspect | Design |
| --- | --- |
| **Purpose** | Types & checklists |
| **Relationships** | Tenant catalog |
| **Indexes** | Unique codes |
| **Constraints** | — |
| **Retention** | Soft retire |
| **Expected growth** | Low–Medium |

### 12.2 `equipment.assets`

| Aspect | Design |
| --- | --- |
| **Purpose** | Asset registry |
| **Relationships** | Type FK; logical company/project/person |
| **Indexes** | Unique `(tenant_id, asset_tag)` partial; `(tenant_id, class, status)`; `(tenant_id, project_id)` assignment; serial unique partial |
| **Constraints** | Class/status checks; profile JSONB |
| **Retention** | Soft retire long |
| **Expected growth** | High |

### 12.3 `equipment.asset_assignments` / `equipment.custody_transfers` / `equipment.qr_bindings` / `equipment.photo_refs`

| Aspect | Design |
| --- | --- |
| **Purpose** | Assignment, custody, QR, photos |
| **Relationships** | FK asset |
| **Indexes** | QR code unique; `(asset_id, assigned_at DESC)` |
| **Constraints** | One active QR binding |
| **Retention** | History retained |
| **Expected growth** | High |

### 12.4 `equipment.inspections` (+ item results, attachments, sig refs)

| Aspect | Design |
| --- | --- |
| **Purpose** | Pre-use & periodic inspections |
| **Relationships** | FK asset; checklist |
| **Indexes** | `(tenant_id, asset_id, created_at DESC)`; `(tenant_id, kind, status)`; due queries |
| **Constraints** | Kind/status |
| **Retention** | Long; partition by created_at when huge |
| **Expected growth** | Very High |

### 12.5 `equipment.deficiencies` / `equipment.maintenance_orders` (+ children)

| Aspect | Design |
| --- | --- |
| **Purpose** | Deficiencies & maintenance |
| **Relationships** | FK asset; links between |
| **Indexes** | `(asset_id, status)`; `(tenant_id, due_at)` |
| **Constraints** | Severity/status |
| **Retention** | Long history |
| **Expected growth** | High |

### 12.6 `equipment.certification_records`

| Aspect | Design |
| --- | --- |
| **Purpose** | Cert validity records |
| **Relationships** | FK asset; logical document/file ids |
| **Indexes** | `(asset_id)`; `(tenant_id, expires_at)` |
| **Constraints** | Period checks |
| **Retention** | Long |
| **Expected growth** | High |

### 12.7 `equipment.binder_templates` / `equipment.equipment_binders` / `equipment.binder_sections` / `equipment.binder_section_items`

| Aspect | Design |
| --- | --- |
| **Purpose** | Tower & self-erect binders |
| **Relationships** | Binder FK asset; sections/items |
| **Indexes** | Unique `(asset_id, binder_kind)` active; section completeness |
| **Constraints** | Kind check |
| **Retention** | With asset |
| **Expected growth** | Medium (crane subset) |

### 12.8 `equipment.readiness_snapshots` / `equipment.reporting_projections`

| Aspect | Design |
| --- | --- |
| **Purpose** | Derived readiness cache & reports |
| **Relationships** | 1:1 asset snapshot |
| **Indexes** | `(tenant_id, readiness_state)` |
| **Constraints** | Rebuildable |
| **Retention** | Ephemeral snapshot ok |
| **Expected growth** | High |

---

## 13. Documents Schema Tables

### 13.1 `documents.documents`

| Aspect | Design |
| --- | --- |
| **Purpose** | Logical controlled document |
| **Relationships** | Versions children |
| **Indexes** | Unique `(tenant_id, document_code)` partial; `(tenant_id, category, status)` |
| **Constraints** | Category/status |
| **Retention** | Soft archive/retire long |
| **Expected growth** | High |

### 13.2 `documents.document_versions`

| Aspect | Design |
| --- | --- |
| **Purpose** | Immutable versions |
| **Relationships** | FK document; logical file_object_id |
| **Indexes** | Unique `(document_id, version_number)`; `(tenant_id, state)`; effective `(document_id, effective_from, effective_to)` |
| **Constraints** | Immutability of published enforced by app; checksum NOT NULL on publish |
| **Retention** | Forever relative to doc retention |
| **Expected growth** | High |

### 13.3 `documents.document_templates`

| Aspect | Design |
| --- | --- |
| **Purpose** | Doc/form templates |
| **Relationships** | Tenant |
| **Indexes** | `(tenant_id, kind)` |
| **Constraints** | — |
| **Retention** | Soft retire |
| **Expected growth** | Medium |

### 13.4 `documents.review_cycles` / `documents.approval_cases` (+ steps/comments/decisions)

| Aspect | Design |
| --- | --- |
| **Purpose** | Review/approval |
| **Relationships** | FK version |
| **Indexes** | `(version_id, status)` |
| **Constraints** | One active approval per version |
| **Retention** | With version |
| **Expected growth** | High |

### 13.5 `documents.assignments` / `documents.acknowledgement_requests` / `documents.acknowledgements`

| Aspect | Design |
| --- | --- |
| **Purpose** | Assign & ack |
| **Relationships** | FK version; logical person; signature_package_id |
| **Indexes** | `(person_id, status)`; unique ack `(version_id, person_id)` |
| **Constraints** | — |
| **Retention** | Long (compliance) |
| **Expected growth** | Very High |

### 13.6 `documents.distribution_lists` / `documents.retention_policies` / `documents.legal_holds` / `documents.disposal_records`

| Aspect | Design |
| --- | --- |
| **Purpose** | Distribution & retention |
| **Relationships** | Holds→documents |
| **Indexes** | Policy by category; holds by document |
| **Constraints** | — |
| **Retention** | Holds until released; disposal records forever |
| **Expected growth** | Medium |

### 13.7 `documents.qr_sign_targets` / `documents.search_projections`

| Aspect | Design |
| --- | --- |
| **Purpose** | QR targets; FTS projection |
| **Relationships** | Version logical |
| **Indexes** | QR unique; FTS gin on projection |
| **Constraints** | — |
| **Retention** | Targets expire; search rebuildable |
| **Expected growth** | High |

---

## 14. Training Schema Tables

### 14.1 `training.courses` / `training.competency_definitions` / `training.evaluation_definitions`

| Aspect | Design |
| --- | --- |
| **Purpose** | Catalog |
| **Relationships** | Links course↔competency; eval defs |
| **Indexes** | Unique codes; kind orientation flag |
| **Constraints** | — |
| **Retention** | Soft retire |
| **Expected growth** | Medium |

### 14.2 `training.evaluation_attempts`

| Aspect | Design |
| --- | --- |
| **Purpose** | Evaluation results |
| **Relationships** | FK definition; logical person; signature ref |
| **Indexes** | `(person_id, definition_id, created_at DESC)` |
| **Constraints** | Outcome check |
| **Retention** | Long |
| **Expected growth** | High |

### 14.3 `training.requirements` / `training.requirement_scopes`

| Aspect | Design |
| --- | --- |
| **Purpose** | Mandatory rules |
| **Relationships** | Scope dimensions JSONB+columns |
| **Indexes** | `(tenant_id, project_id)`; `(trade_code)`; `(role)` |
| **Constraints** | — |
| **Retention** | Soft remove |
| **Expected growth** | Medium–High |

### 14.4 `training.assignments`

| Aspect | Design |
| --- | --- |
| **Purpose** | Person obligations |
| **Relationships** | Logical person/course; requirement source |
| **Indexes** | `(tenant_id, person_id, status)`; `(due_at)`; unique open `(person_id, course_id)` partial |
| **Constraints** | Status |
| **Retention** | Soft cancel; retain completed |
| **Expected growth** | Very High |

### 14.5 `training.completions`

| Aspect | Design |
| --- | --- |
| **Purpose** | Authoritative completions/certificates |
| **Relationships** | Logical person/course; evidence file/doc; attempt id |
| **Indexes** | `(person_id, course_id, valid_to)`; `(tenant_id, valid_to)` expiry jobs; unique current valid partial optional |
| **Constraints** | valid_from ≤ valid_to |
| **Retention** | Long / forever for COR |
| **Expected growth** | Very High |

### 14.6 `training.waivers` / `training.renewal_cases`

| Aspect | Design |
| --- | --- |
| **Purpose** | Waivers & renewals |
| **Relationships** | Person/completion links |
| **Indexes** | `(status, due_at)` |
| **Constraints** | — |
| **Retention** | Waivers audited long |
| **Expected growth** | Medium |

### 14.7 `training.toolbox_library_items`

| Aspect | Design |
| --- | --- |
| **Purpose** | Toolbox content library |
| **Relationships** | Tenant catalog |
| **Indexes** | Tags; unique code |
| **Constraints** | — |
| **Retention** | Soft retire |
| **Expected growth** | Medium |

### 14.8 `training.matrix_projections` / `training.reporting_projections`

| Aspect | Design |
| --- | --- |
| **Purpose** | Matrix & reports |
| **Relationships** | Rebuildable |
| **Indexes** | `(tenant_id, project_id, person_id)` |
| **Constraints** | — |
| **Retention** | Rebuild |
| **Expected growth** | High |

---

## 15. COR Schema Tables

### 15.1 `cor.audit_frameworks` / `cor.framework_elements` (+ guidelines/score rules)

| Aspect | Design |
| --- | --- |
| **Purpose** | BCCSA COR / SECOR / regional packs |
| **Relationships** | Elements FK framework |
| **Indexes** | Unique `(family, version)`; element `(framework_id, element_code)` |
| **Constraints** | — |
| **Retention** | Forever versioned |
| **Expected growth** | Low (content), Medium (custom) |

### 15.2 `cor.readiness_profiles` / `cor.coverage_cells` / `cor.evidence_mappings` / `cor.gap_items`

| Aspect | Design |
| --- | --- |
| **Purpose** | Continuous readiness |
| **Relationships** | Profile per subject+framework; mappings provenance JSON/columns |
| **Indexes** | Unique `(tenant_id, subject_type, subject_id, framework_id)`; gaps `(status, due_at)` |
| **Constraints** | Coverage status check |
| **Retention** | Long |
| **Expected growth** | High |

### 15.3 `cor.audit_plans` / `cor.audit_engagements`

| Aspect | Design |
| --- | --- |
| **Purpose** | Planning & engagements |
| **Relationships** | Engagement FK plan optional; framework version pin |
| **Indexes** | `(tenant_id, status)`; `(framework_id)` |
| **Constraints** | Type/status |
| **Retention** | Forever historical |
| **Expected growth** | Medium |

### 15.4 `cor.interviews` / `cor.observations` / `cor.audit_findings` / `cor.audit_corrective_actions`

| Aspect | Design |
| --- | --- |
| **Purpose** | Fieldwork artifacts |
| **Relationships** | FK engagement; logical person; optional safety_ca_id |
| **Indexes** | `(engagement_id)`; findings `(status, severity)` |
| **Constraints** | — |
| **Retention** | Forever with engagement |
| **Expected growth** | Medium–High |

### 15.5 `cor.scorecards` / `cor.evidence_packages` / `cor.package_items` / `cor.audit_reports` / `cor.engagement_snapshots`

| Aspect | Design |
| --- | --- |
| **Purpose** | Scoring, packages, reports, history snapshots |
| **Relationships** | FK engagement; file refs logical |
| **Indexes** | Package `(status)`; snapshot unique engagement |
| **Constraints** | Immutable snapshot |
| **Retention** | Forever |
| **Expected growth** | Medium |

### 15.6 `cor.dashboard_projections` / `cor.analytics_projections`

| Aspect | Design |
| --- | --- |
| **Purpose** | COR UI projections |
| **Relationships** | Rebuildable |
| **Indexes** | By subject |
| **Constraints** | — |
| **Retention** | Rebuild |
| **Expected growth** | Medium |

---

## 16. Signatures Schema Tables

### 16.1 `signatures.signing_policies`

| Aspect | Design |
| --- | --- |
| **Purpose** | Assurance/order/expiry rules |
| **Relationships** | Tenant |
| **Indexes** | Unique `(tenant_id, process_type, subject_type)` |
| **Constraints** | Assurance checks |
| **Retention** | Version/retire |
| **Expected growth** | Low |

### 16.2 `signatures.signature_packages`

| Aspect | Design |
| --- | --- |
| **Purpose** | Package aggregate |
| **Relationships** | Subject binding columns; children slots |
| **Indexes** | `(tenant_id, status)`; `(subject_type, subject_id)`; `(expires_at)` |
| **Constraints** | Status; pinned version fields |
| **Retention** | Long / forever for evidence |
| **Expected growth** | Very High — partition by created_at |

### 16.3 `signatures.signer_slots` / `signatures.captured_signatures` / `signatures.identity_assurance_records`

| Aspect | Design |
| --- | --- |
| **Purpose** | Slots, seals, IDV snapshots |
| **Relationships** | FK package |
| **Indexes** | `(package_id, order_index)`; `(assignee_user_id, status)` |
| **Constraints** | Slot status |
| **Retention** | With package |
| **Expected growth** | Very High |

### 16.4 `signatures.magic_links` / `signatures.qr_sign_sessions`

| Aspect | Design |
| --- | --- |
| **Purpose** | Guest/QR access |
| **Relationships** | FK package/slot |
| **Indexes** | Unique token_hash; `(expires_at)` |
| **Constraints** | Store hash only |
| **Retention** | Hard delete soon after expiry (e.g., 30–90d) keeping package evidence |
| **Expected growth** | High |

### 16.5 `signatures.evidence_certificates`

| Aspect | Design |
| --- | --- |
| **Purpose** | Certificate artifacts metadata |
| **Relationships** | Unique package_id; file_object_id |
| **Indexes** | `(package_id)` unique |
| **Constraints** | Manifest hash NOT NULL |
| **Retention** | Forever with package |
| **Expected growth** | Very High |

### 16.6 `signatures.audit_projections`

| Aspect | Design |
| --- | --- |
| **Purpose** | Signature history search |
| **Relationships** | Rebuildable from packages |
| **Indexes** | `(tenant_id, occurred_at)` |
| **Constraints** | — |
| **Retention** | Align package retention |
| **Expected growth** | High |

---

## 17. Notifications Schema Tables

### 17.1 `notifications.templates` / `notifications.template_variants`

| Aspect | Design |
| --- | --- |
| **Purpose** | Multi-channel templates |
| **Relationships** | Variant FK template |
| **Indexes** | Unique `(tenant_id, code)` / channel+locale |
| **Constraints** | — |
| **Retention** | Soft retire |
| **Expected growth** | Low–Medium |

### 17.2 `notifications.delivery_rules` / `notifications.escalation_policies`

| Aspect | Design |
| --- | --- |
| **Purpose** | Routing & escalation |
| **Relationships** | Policy refs |
| **Indexes** | `(event_type)` |
| **Constraints** | Priority checks |
| **Retention** | Soft |
| **Expected growth** | Low |

### 17.3 `notifications.preferences` / `notifications.subscriptions` / `notifications.channel_connectors`

| Aspect | Design |
| --- | --- |
| **Purpose** | Prefs, topics, Teams/WhatsApp connectors |
| **Relationships** | Logical user_id |
| **Indexes** | Unique user prefs; connector `(tenant_id, type)` |
| **Constraints** | Opt-in flags for WhatsApp/SMS |
| **Retention** | Soft; consent records longer |
| **Expected growth** | Medium |

### 17.4 `notifications.notifications`

| Aspect | Design |
| --- | --- |
| **Purpose** | Inbox messages |
| **Relationships** | Recipient; dedup_key |
| **Indexes** | `(tenant_id, recipient_user_id, read_status, created_at DESC)`; unique `(tenant_id, dedup_key)` partial |
| **Constraints** | Priority/status |
| **Retention** | 90–180 days typical then hard; **partition monthly** |
| **Expected growth** | Extremely High |

### 17.5 `notifications.delivery_jobs` / `notifications.delivery_attempts`

| Aspect | Design |
| --- | --- |
| **Purpose** | Queue & retries |
| **Relationships** | FK notification |
| **Indexes** | `(status, available_at)`; attempts `(job_id, attempt_no)` unique |
| **Constraints** | Error class check |
| **Retention** | 30–90 days; DLQ longer; partition |
| **Expected growth** | Extremely High |

### 17.6 `notifications.digest_schedules` / `notifications.digest_batches` / `notifications.escalation_executions`

| Aspect | Design |
| --- | --- |
| **Purpose** | Digests & escalation execution |
| **Relationships** | Batch items link notifications |
| **Indexes** | Schedule active; batch `(scheduled_for)` |
| **Constraints** | — |
| **Retention** | 90 days |
| **Expected growth** | High |

### 17.7 `notifications.reporting_projections`

| Aspect | Design |
| --- | --- |
| **Purpose** | Delivery metrics |
| **Relationships** | Rebuildable |
| **Indexes** | By day/channel |
| **Constraints** | — |
| **Retention** | 1 year |
| **Expected growth** | Medium |

---

## 18. Analytics Metadata Schema Tables

> Facts live in ClickHouse. Postgres holds **metadata only**.

### 18.1 `analytics.metric_definitions`

| Aspect | Design |
| --- | --- |
| **Purpose** | Metric catalog |
| **Relationships** | — |
| **Indexes** | Unique `metric_key` |
| **Constraints** | Type check |
| **Retention** | Forever / retire flag |
| **Expected growth** | Low |

### 18.2 `analytics.dashboard_definitions` / `analytics.dashboard_widgets`

| Aspect | Design |
| --- | --- |
| **Purpose** | Dashboard configs |
| **Relationships** | Widget FK dashboard; metric_key logical |
| **Indexes** | `(tenant_id, persona)` |
| **Constraints** | Viz type |
| **Retention** | Soft |
| **Expected growth** | Low–Medium |

### 18.3 `analytics.report_definitions` / `analytics.analytics_subscriptions`

| Aspect | Design |
| --- | --- |
| **Purpose** | Custom reports & schedules |
| **Relationships** | Subscription→report |
| **Indexes** | `(tenant_id, owner_user_id)` |
| **Constraints** | — |
| **Retention** | Soft |
| **Expected growth** | Medium |

### 18.4 `analytics.export_jobs`

| Aspect | Design |
| --- | --- |
| **Purpose** | Async exports |
| **Relationships** | Logical file_object_id |
| **Indexes** | `(tenant_id, status, created_at)` |
| **Constraints** | Format/status |
| **Retention** | 30–90 days metadata; files per policy |
| **Expected growth** | Medium–High — partition optional |

### 18.5 `analytics.ingest_checkpoints`

| Aspect | Design |
| --- | --- |
| **Purpose** | Pipeline watermarks |
| **Relationships** | — |
| **Indexes** | Unique consumer name |
| **Constraints** | — |
| **Retention** | Forever small |
| **Expected growth** | Low |

---

## 19. Workflows Schema Tables

### 19.1 `workflows.workflow_definitions`

| Aspect | Design |
| --- | --- |
| **Purpose** | Tenant-visible workflow metadata bound to Temporal types |
| **Relationships** | Logical Temporal workflow type name |
| **Indexes** | Unique `(tenant_id, code)` |
| **Constraints** | — |
| **Retention** | Soft retire |
| **Expected growth** | Low–Medium |

### 19.2 `workflows.workflow_instances`

| Aspect | Design |
| --- | --- |
| **Purpose** | Tracking/correlation for running processes |
| **Relationships** | FK definition optional; subject refs logical; temporal_run_id |
| **Indexes** | `(tenant_id, status)`; `(subject_type, subject_id)`; unique `temporal_run_id` |
| **Constraints** | Status check |
| **Retention** | Closed instances 1–2 years then archive; **partition by started_at** |
| **Expected growth** | Very High |

### 19.3 `workflows.workflow_milestones` / `workflows.escalation_steps`

| Aspect | Design |
| --- | --- |
| **Purpose** | Visibility & escalation config copies |
| **Relationships** | FK instance / definition |
| **Indexes** | `(instance_id, occurred_at)` |
| **Constraints** | — |
| **Retention** | With instance |
| **Expected growth** | Very High |

> Temporal server DB is separate infrastructure—not modeled here.

---

## 20. Cross-Module Reference Map (Logical Only)

| From | To | Column examples |
| --- | --- | --- |
| Almost all | `core.tenants` | `tenant_id` |
| Safety/Equipment/Training/… | `projects.projects` | `project_id` |
| Many | `people.persons` | `person_id` |
| Participants/employments | `core.companies` | `company_id` |
| Memberships | `projects` + `people` | `project_id`, `person_id` |
| Attachments | `core.file_objects` | `file_object_id` |
| Acks/inspections | `signatures.signature_packages` | `signature_package_id` |
| Procedure bindings | `documents.document_versions` | `document_version_id` |
| COR mappings | multi-module | `provenance_module`, `provenance_id` |

No PostgreSQL FKs across these boundaries.

---

## 21. Retention Policy Summary

| Class | Examples | Guidance |
| --- | --- | --- |
| **Security audit** | `core.audit_entries` | 7+ years / legal |
| **Compliance evidence** | activities, completions, packages, seals, docs versions | 7+ years / COR |
| **Operational inbox** | notifications | 90–180 days |
| **Ephemeral** | sessions, magic links, idempotency, outbox | days–weeks |
| **PHI/PII sensitive** | medical, emergency | Minimize; strict purge rules |
| **Rebuildable projections** | dashboards/matrix | Disposable |
| **Commercial** | licenses, billing | Life of account + statutory |

Exact durations are tenant/legal settings—not hardcoded in this architecture.

---

## 22. Growth & Capacity Planning (Order of Magnitude)

| Tier | Row scale (mature multi-tenant) | Examples |
| --- | --- | --- |
| Low | < 1M | tenants, frameworks, metric catalog |
| Medium | 1–20M | projects, roles, templates |
| High | 20–200M | persons, assets, memberships |
| Very High | 200M–billions | activities, inspections, notifications, audit, signatures, completions |

Partition + archival mandatory for Very High class before production scale-up.

---

## 23. Security Checklist (Database)

- [ ] RLS enabled + forced on app role  
- [ ] `app.tenant_id` set per transaction  
- [ ] Least-privilege DB roles  
- [ ] Audit table insert-only grants  
- [ ] No raw API keys / magic secrets  
- [ ] Encryption at rest (provider) + TLS  
- [ ] PITR tested  
- [ ] PHI tables access reviewed  
- [ ] Cross-schema FK absent by policy lint  

---

## 24. Success Criteria

The PostgreSQL architecture succeeds when:

1. Module schemas evolve independently without cross-schema FK entanglement.  
2. Tenant isolation holds under RLS even if app bugs omit a predicate.  
3. Hot tables partition cleanly with predictable retention.  
4. Audit and evidence tables are immutable and durable.  
5. JSONB accelerates flexibility without destroying relational integrity.  
6. Analytics/ClickHouse relieve OLTP from trend queries.  
7. PostGIS and read replicas can be added without redesigning tenancy.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | PostgreSQL Enterprise Architecture | Complete OLTP database architecture (no SQL DDL) |

---

*End of PostgreSQL Enterprise Database Architecture*
