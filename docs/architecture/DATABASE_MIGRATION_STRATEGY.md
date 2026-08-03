# Proven — Database Migration Strategy

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Database Migration Strategy |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Database / Platform Architecture |
| **Audience** | Backend, SRE, Module Owners, Security |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [PostgreSQL Architecture](./POSTGRESQL_ARCHITECTURE.md), [Rust Crate Catalog](./RUST_CRATE_CATALOG.md), [Repository Plan](./REPOSITORY_PLAN.md), [GitHub Repository Design](../engineering/GITHUB_REPOSITORY.md), [Security Architecture](./SECURITY_ARCHITECTURE.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document defines Proven’s **PostgreSQL migration strategy**: naming, versioning, rollback philosophy, indexes, seed vs reference data, development/testing/production practices, tenant-oriented data migrations, and future expansion.

**Hard rules**

1. **Expand/contract** for zero/low-downtime deploys — never combine incompatible API + destructive DDL in one step.  
2. **One schema owner per change** — no cross-module schema coupling via migrations.  
3. **No cross-schema foreign keys** — UUID references only ([PostgreSQL Architecture](./POSTGRESQL_ARCHITECTURE.md)).  
4. **Forward-first in production** — prefer roll-forward; down migrations are for local/dev only unless explicitly rehearsed.  
5. **This document contains no SQL** — structural and process design only.

**Documentation only.**

---

## 2. Goals

| Goal | Meaning |
| --- | --- |
| Safety | RLS, tenancy, and audit integrity preserved across changes |
| Compatibility | Old and new API binaries can coexist during rolling deploy |
| Traceability | Every prod change maps to a versioned migration file + release |
| Speed | Local/CI migrate quickly on ephemeral databases |
| Isolation | Module teams ship independently without stepping on each other |

---

## 3. Repository Layout (Logical)

```text
db/
├── migrations/
│   ├── platform/           # outbox, migration aids, shared platform DDL
│   ├── core/
│   ├── projects/
│   ├── people/
│   ├── safety/
│   ├── equipment/
│   ├── documents/
│   ├── signatures/
│   ├── notifications/
│   ├── training/
│   ├── cor/
│   ├── analytics/          # Postgres analytics metadata only (not ClickHouse)
│   ├── admin/
│   ├── integrations/
│   └── workflows/
├── seeds/
│   ├── local/              # developer convenience
│   └── ci/                 # minimal deterministic fixtures
├── reference/              # versioned system reference packs (COR frameworks, hazard libraries templates)
├── scripts/                # migrate, lint, drift-check wrappers (no ad-hoc prod SQL)
└── README.md
```

Tool choice (goose / atlas / sqlx / sqitch) is an ADR later; **process** in this document is tool-agnostic.

Migrator role: `proven_migrator` (DDL). App role: `proven_app` (DML + forced RLS). Never migrate as the app role.

---

## 4. Migration Naming

### 4.1 File name pattern

```text
{utc_timestamp}_{schema}_{slug}.sql
```

| Part | Rule | Example intent |
| --- | --- | --- |
| `utc_timestamp` | `YYYYMMDDHHMMSS` UTC; monotonically increasing within schema chain | Ordering |
| `schema` | Module schema owner (`core`, `safety`, …) | Ownership lint |
| `slug` | kebab-case verb-noun | `add-corrective-action-due-at` |

**Optional prefix** for multi-phase expand/contract:

```text
{timestamp}_{schema}_{phase}-{slug}.sql
```

Phases: `expand`, `backfill` *(job, not always DDL)*, `contract`.

### 4.2 Content metadata (header comments — conceptual)

Each migration declares:

- Schema owner  
- Author / PR  
- Expand | Contract | Idempotent note  
- Lock risk (none / short / requires maintenance window)  
- Rollback posture (roll-forward only | local down available)  

### 4.3 Forbidden naming

- Unscoped `update-stuff`  
- Mixing two schemas in one file  
- Reusing timestamps  
- Encoding ticket-only names without schema (`JIRA-1234.sql`)

---

## 5. Versioning

### 5.1 Migration version identity

| Concept | Definition |
| --- | --- |
| **Version** | The timestamp (and schema path) applied by the migrator ledger |
| **Ledger** | Tool-managed table (e.g. conceptually `platform.schema_migrations` / tool default) recording applied versions |
| **Chain** | Ordered list per schema directory; global apply order = timestamp order across schemas **or** explicit dependency manifest |

### 5.2 Apply order

**Recommended:** single global timestamp ordering across all `db/migrations/**` so interleaving is deterministic. Schema folders are for **ownership**, not separate version counters that can collide.

Alternative (ADR): per-schema version sequences with a dependency graph—only if tooling supports it cleanly.

### 5.3 Relationship to product SemVer

| Product bump | Migration expectation |
| --- | --- |
| PATCH | Additive indexes, bugfix constraints, non-breaking expands |
| MINOR | New tables/columns (expand); new modules schemas |
| MAJOR | Contract drops after deprecation window; rare hard cuts |

Migrations are **not** SemVer themselves; Git tags record which migration head shipped.

### 5.4 Drift detection

CI/prod job: compare ledger + filesystem head. Fail deploy on unexpected drift. Schema dump diff optional for review.

---

## 6. Rollback

### 6.1 Philosophy

| Environment | Policy |
| --- | --- |
| **Local / CI** | Down migrations optional for convenience; recreate DB preferred |
| **Staging / Production** | **Roll forward**; restore from backup only for catastrophic failure |

### 6.2 Why forward-first

- Downs that drop columns destroy data expanded apps may still need.  
- Rolling deploys mean mixed binaries; downs race with live traffic.  
- Backfills are often irreversible without backup.

### 6.3 Rollback playbooks

| Situation | Action |
| --- | --- |
| Bad expand (nullable column) | Ship fix-forward migration; feature-flag off usage |
| Bad index | Create corrected index concurrently; drop bad index in follow-up |
| Failed mid-migration | Transactional DDL where possible; otherwise repair migration + runbook |
| Catastrophic corruption | PITR / backup restore (SRE); incident process |

### 6.4 “Down” files

If tooling generates downs:

- Allowed for **dev** recreate workflows.  
- **Not** executed automatically in production pipelines.  
- Contract downs that drop data require explicit human approval checklist—even in staging.

---

## 7. Expand / Contract Pattern

```text
Phase 1 EXPAND     Add new structures (nullable columns, new tables, new indexes)
                   Deploy API that dual-reads / dual-writes as needed
Phase 2 BACKFILL   Asynchronous job or batched migrator updates existing rows
Phase 3 SWITCH     API prefers new shape; old path deprecated
Phase 4 CONTRACT   Remove old columns/tables/indexes after observation window
```

### 7.1 Rules

1. Expand migrations must leave **previous app version** functional.  
2. Backfills run as **jobs** (Temporal/worker/API) with idempotency—not multi-hour locks in the migration transaction when avoidable.  
3. Contract only after metrics show zero reads of old shape.  
4. Renames = add new + backfill + switch + drop old (never rename in place under load without expansion).

### 7.2 Lock risk classes

| Class | Examples | Handling |
| --- | --- | --- |
| **Safe** | New table, new nullable column | Normal migrate in deploy |
| **Careful** | Index build | Prefer concurrent index strategy; may be separate migration |
| **Dangerous** | Rewrite large table, tight locks | Maintenance window + rehearsal on staging restore |

Label PRs `risk:data-migration` when careful/dangerous.

---

## 8. Indexes

### 8.1 Principles

- Indexes ship in migrations owned by the table’s schema.  
- Prefer indexes supporting known query paths (tenant + filter + time).  
- Partial indexes for soft-delete / active rows where documented.  
- GIN for FTS/`jsonb` only where search architecture requires.  
- Avoid speculative indexes “just in case.”

### 8.2 Creation strategy

| Approach | When |
| --- | --- |
| Transactional create | Small tables / empty new tables |
| Concurrent create pattern | Large live tables (tooling/process that avoids long exclusive locks) |

Follow-up migration may “validate/attach” if using concurrent patterns—document in migration header.

### 8.3 Dropping indexes

- Contract phase only.  
- Confirm query plans on staging with production-like stats.  
- Drop unused indexes to reduce write amplification.

### 8.4 Review checklist

- Includes `tenant_id` leading or matching RLS/query pattern when selective.  
- Unique constraints are **partial** where soft-delete reuse requires.  
- No unique across tenants accidentally.

---

## 9. Seed Data

### 9.1 Definition

**Seeds** = non-production or bootstrap rows for **local/CI** ergonomics (demo tenant, sample project). Not a substitute for reference packs.

### 9.2 Rules

| Rule | Detail |
| --- | --- |
| Location | `db/seeds/local`, `db/seeds/ci` |
| PII | **Forbidden** — no real customer data |
| Idempotent | Re-runnable or reset-friendly |
| Production | **Not applied** by default prod migrate |
| Bootstrap | Tenant onboarding creates real tenants via Core APIs/workflows—not seed scripts in prod |

### 9.3 Dev reset

`scripts/db/reset-local`: drop/recreate → migrate → seed. Never pointed at shared staging without guardrails.

---

## 10. Reference Data

### 10.1 Definition

**Reference data** = system- or pack-level catalogs required for product behavior: COR/SECOR framework elements, default hazard/control libraries, activity type definitions, permission catalogs, metric catalog entries.

### 10.2 Delivery mechanisms

| Mechanism | Use |
| --- | --- |
| **Migration-bundled** | Tiny, immutable platform catalogs (e.g., permission codes) tightly coupled to code |
| **Versioned reference packs** in `db/reference/` + loader command | COR packs, hazard libraries—versioned, reviewable, upgradable |
| **Admin/module APIs** | Tenant-custom extensions after pack install |

### 10.3 Rules

1. Reference packs are **content versioned** (pack semver) independent of migration timestamps.  
2. Upgrading a pack is an explicit command/workflow with audit—not a silent overwrite of tenant customizations.  
3. Tenant-owned copies vs system templates: document merge policy per module.  
4. No reference load that bypasses AuthZ in production multi-tenant contexts (platform bootstrap role only for system packs).

---

## 11. Development

| Practice | Detail |
| --- | --- |
| **Create** | New file via generator script ensuring timestamp uniqueness |
| **Own one schema** | PR touches one schema folder unless platform review |
| **Run** | Compose Postgres → migrate up |
| **Iterate** | Prefer new migration over editing already-merged migrations |
| **Editing history** | Allowed only if never applied to shared envs; otherwise fix-forward |
| **RLS** | Enable/force policies in same expand that creates tenant tables |
| **Config** | `.env` / local config for `DATABASE_URL`; migrator credentials separate from app |

Developers do not hand-apply DDL in shared databases.

---

## 12. Testing

### 12.1 CI pipeline (`ci-db`)

| Check | Purpose |
| --- | --- |
| **Lint names** | Pattern + schema ownership |
| **Migrate from empty** | Full chain applies cleanly |
| **Migrate twice** | Idempotent ledger / no double-apply |
| **Expand compatibility** | Optional: previous release binary against new schema smoke |
| **Policy lint** | No cross-schema FK; RLS present on tenant tables (heuristic) |
| **Down (optional)** | Local-only job; not required green for merge if forward-only |

### 12.2 Integration tests

Module integ tests run against migrated ephemeral DB (`proven-test-support`). Tests must not assume seed data unless explicitly loaded.

### 12.3 Staging rehearsal

Before dangerous prod migrations: restore recent prod-like backup to staging → migrate → run smoke + critical queries → record timing/locks.

---

## 13. Production

### 13.1 Pipeline

```text
Release tag / main deploy
  → pre-migrate checks (drift, backup freshness)
  → run migrator (proven_migrator) expand/safe migrations
  → deploy API/workers (compatible)
  → verify health + smoke
  → (later) backfill jobs
  → (later release) contract migrations
```

### 13.2 Gates

- Automated migrate in deploy—**no** SSH improvisation.  
- Maintenance window approval for dangerous class.  
- `risk:data-migration` + CODEOWNERS on `/db/`.  
- Backup / PITR confirmed before contract drops.  
- Feature flags for new columns usage when risk warrants.

### 13.3 Observability

- Migration duration, lock waits, replication lag (if any).  
- App error rates during expand window.  
- Alert on migrator failure; block subsequent app deploy if configured as coupled step.

### 13.4 Hotfix

Patch migrations allowed; still expand/contract. Hotfix binary must understand schema before/after expand.

---

## 14. Tenant Migration

Clarifies two different meanings:

### 14.1 Schema migrations (all tenants)

Proven uses **shared schemas + `tenant_id`**. DDL applies once globally; all tenants inherit structure. There is **no per-tenant DDL version**.

### 14.2 Tenant data migrations / moves

| Scenario | Strategy |
| --- | --- |
| **Backfill per tenant** | Batched jobs keyed by `tenant_id`; idempotent; rate-limited |
| **Onboarding** | `TenantAdminOnboardingWorkflow` + Core APIs—not SQL seeds |
| **Pack install** | Reference pack loader scoped to tenant with audit |
| **Export/import tenant** | Future: documented ETL via APIs/events—not direct pg_dump into prod |
| **Tenant split/merge** | Rare; project-level playbook + dual-write period + AuthZ rewrite; ADR required |
| **Region move** | Future residency: dump/restore or logical replication playbook + DNS/cutover; RLS verified post-move |

### 14.3 Rules for tenant data jobs

1. Set tenant context per batch (RLS).  
2. Emit audit for bulk substantive changes.  
3. Checkpoint progress; resume safely.  
4. Never run cross-tenant updates in one transaction without platform review.

---

## 15. Future Expansion

| Expansion | Migration implication |
| --- | --- |
| **New module schema** | New folder under `db/migrations/<module>/`; first migration creates schema + privileges + RLS defaults |
| **Partitioning** | Expand: new partitioned table → backfill → switch reads/writes → contract old; attach partitions via automation |
| **PostGIS** | Optional extension migration in `platform` with SRE review |
| **Read replicas** | No DDL divergence; migrator targets primary only |
| **ClickHouse** | Separate analytics migrator/versioning—not Postgres chain ([Data Warehouse](./DATA_WAREHOUSE_ARCHITECTURE.md)) |
| **Multi-region DB** | Per-region migration trains; reference packs replicated; avoid split-brain ledger |
| **Table rewrite tools** | Online schema change tools via ADR if lock budgets fail |
| **Crypto key rotation** | App-level; may need nullable new columns expand pattern |

---

## 16. Roles & Ownership

| Role | Responsibility |
| --- | --- |
| Module owner | Migrations in their schema folder |
| Platform DB | `platform` schema, tooling, lint rules |
| Security | RLS policy changes, privilege grants |
| SRE | Prod migrate execution, backups, windows |
| CODEOWNERS | `/db/` → backend; schema hotspots as needed |

---

## 17. PR / Review Checklist

- [ ] Single schema owner (or platform exception)  
- [ ] Naming matches pattern  
- [ ] Expand/contract phase identified  
- [ ] Lock risk labeled  
- [ ] Indexes justified  
- [ ] RLS/privileges updated for new tenant tables  
- [ ] No cross-schema FK  
- [ ] No prod seed  
- [ ] Backfill plan if data rewrite needed  
- [ ] Roll-forward plan documented  
- [ ] `risk:data-migration` if careful/dangerous  

---

## 18. Success Criteria

1. Empty database reaches current head via one documented migrate path.  
2. Rolling deploys never require simultaneous incompatible schema + binary.  
3. Production rollback is roll-forward or restore—not surprise downs.  
4. Seeds never pollute production; reference packs are versioned and auditable.  
5. Tenant isolation holds before and after every migration.  
6. Future modules/partitions/regions extend the same conventions.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Database Migration Architecture | Strategy without SQL |

---

*End of Database Migration Strategy*
