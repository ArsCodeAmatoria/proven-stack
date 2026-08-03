# Proven — Data Warehouse & Analytics Architecture (ClickHouse)

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Data Warehouse / Analytics Architecture |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Data Warehouse / Analytics Architecture |
| **Audience** | Analytics Engineering, Backend, SRE, Product, Executive reporting |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Analytics Domain](./ANALYTICS_DOMAIN.md), [Event Catalog](./EVENT_CATALOG.md), [Go Workers](./GO_WORKERS_ARCHITECTURE.md), [PostgreSQL](./POSTGRESQL_ARCHITECTURE.md), [Search](./SEARCH_ARCHITECTURE.md), [Security](./SECURITY_ARCHITECTURE.md), [System Architecture](./SYSTEM_ARCHITECTURE.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document defines Proven’s **analytical data warehouse** on **ClickHouse**: dimensional model (dimensions & facts), KPI derivation, domain marts (Safety, Equipment, Workers, Training, COR, Projects), executive dashboards, historical reports, refresh, retention, and exports.

It complements [Analytics Domain](./ANALYTICS_DOMAIN.md) (product catalog, AuthZ presentation, dashboard aggregates). This document is the **warehouse physical/logical design**.

**Hard rules**

1. ClickHouse holds **analytical projections** — not operational SoR.  
2. Ingest from **domain events** (NATS → Go analytics workers); occasional rebuild snapshots.  
3. **AuthZ at query time** via Analytics API (Core scopes)—CH is not a public query surface.  
4. **Minimize PII/PHI** in facts; person-level grain requires Restricted sensitivity + stricter permissions.  
5. Enforcement (readiness, competency gates) never waits on the warehouse.

**Architecture documentation only — no implementation.**

---

## 2. Dual-Store Topology

| Store | Role |
| --- | --- |
| **PostgreSQL OLTP** | Module SoR + `analytics` schema (metric catalog, dashboards, reports, export jobs) |
| **ClickHouse** | Facts, dimension snapshots, rollups, trend queries |
| **R2** | Export artifacts (CSV/XLSX/PDF) |
| **Redis** | Short-lived query/result cache for hot dashboard tiles |
| **Temporal** | Scheduled refresh checks, export workflows, rebuild jobs |

```text
Domain events (outbox → NATS)
        │
        ▼
Go analytics-worker (transform, validate, idempotent insert)
        │
        ▼
ClickHouse: raw facts → MVs / rollup tables
        │
        ▼
Rust Analytics query API (AuthZ + metric catalog)
        │
        ├── Executive / domain dashboards
        ├── Historical / custom reports
        └── ExportJob → Go render → R2
```

---

## 3. Modeling Approach

### 3.1 Style

- **Dimensional model**: conformed dimensions + domain fact tables.  
- **Event-sourced facts** at business grain (one row ≈ one meaningful occurrence).  
- **Periodic snapshot facts** where gauges matter (open CA count, readiness score).  
- **Conformed calendar and tenant** dimensions shared across marts.

### 3.2 Naming

| Pattern | Example |
| --- | --- |
| Dimensions | `dim_*` |
| Facts | `fact_*` |
| Daily rollups | `agg_*_day` |
| Monthly rollups | `agg_*_month` |
| Staging | `stg_*` (optional, short retention) |

Database/database-per-env; tables prefixed or database `proven_analytics`. Tenant isolation via `tenant_id` on every table (mandatory filter).

### 3.3 Engines (Logical Choices)

| Use | Engine pattern |
| --- | --- |
| Append-only events | `MergeTree` / `ReplacingMergeTree` (idempotent `fact_id`) |
| Mutable dimension attrs | `ReplacingMergeTree` ordered by natural key + `updated_at` |
| Aggregating rollups | `SummingMergeTree` / `AggregatingMergeTree` or materialized views into MergeTree |
| Dictionary lookups | ClickHouse **dictionaries** for hot dim attributes (optional) |

Exact engine parameters are implementation detail; this doc fixes **grains and keys**.

---

## 4. Conformed Dimensions

### 4.1 Core Dimensions

| Dimension | Grain / key | Attributes (examples) | Source |
| --- | --- | --- | --- |
| **`dim_tenant`** | `tenant_id` | region_code, status, industry vertical | Core |
| **`dim_date`** | `date` | day, week, month, quarter, year, fiscal_* | Generated |
| **`dim_company`** | `tenant_id, company_id` | name, company_type (GC/sub), status | Core |
| **`dim_org_unit`** | `tenant_id, org_unit_id` | name, parent_id, path | Core |
| **`dim_project`** | `tenant_id, project_id` | code, name, status, region, company_owner_id, activated_at | Projects |
| **`dim_area`** | `tenant_id, area_id` | project_id, name | Projects |
| **`dim_person`** | `tenant_id, person_id` | display_name_hash or limited display, status, primary_trade, workforce_role | People *(PII policy)* |
| **`dim_trade`** | `tenant_id, trade_code` | label | People/Training |
| **`dim_asset`** | `tenant_id, asset_id` | tag, class, type, status | Equipment |
| **`dim_asset_class`** | `asset_class` | label | Equipment |
| **`dim_activity_type`** | `activity_type_code` | FLHA, Toolbox, Inspection, … | Safety |
| **`dim_document_type`** | `document_type` | SWP, SJP, Policy, … | Documents |
| **`dim_course`** | `tenant_id, course_id` | code, title, category | Training |
| **`dim_framework`** | `framework_id, version` | COR/SECOR pack name | COR |
| **`dim_element`** | `framework_id, element_id` | code, title | COR |
| **`dim_metric`** | `metric_key` | type, unit, domain, sensitivity | Analytics catalog (synced) |

### 4.2 Dimension Refresh

- **Type 1** overwrite for most operational attrs (name, status).  
- Preserve **effective dating** only where historically required (optional `dim_project_hist`).  
- Workers upsert dims from events; nightly reconcile from module list APIs for drift.

### 4.3 PII Rules for `dim_person`

- Executive aggregates use `person_id` only when AuthZ allows drill-down.  
- Prefer anonymous counts in default exports.  
- No medical attributes in CH.  
- Display names optional behind Restricted sensitivity class.

---

## 5. Fact Tables

### 5.1 Universal Fact Columns

Every fact includes:

| Column | Purpose |
| --- | --- |
| `fact_id` | Idempotent UUID (from event_id or deterministic hash) |
| `tenant_id` | Isolation |
| `occurred_at` | Domain event time |
| `ingested_at` | Load time |
| `event_type` / `event_version` | Provenance |
| `correlation_id` | Trace |
| `project_id` | Nullable when not applicable |
| `company_id` | Nullable |
| Dimension FKs as needed | — |
| `subject_module`, `subject_type`, `subject_id` | Drill-back to OLTP |

### 5.2 Safety Facts

| Table | Grain | Measures / flags |
| --- | --- | --- |
| **`fact_safety_activity`** | Activity lifecycle event (created/submitted/sealed/closed/voided) | `activity_type`, status_from/to, residual_risk, sealed_flag, duration_to_seal |
| **`fact_corrective_action`** | CA lifecycle event | severity, status, due_at, overdue_flag, aging_days_at_event |
| **`fact_incident`** | Incident opened/updated/closed | severity, type, status |
| **`fact_near_miss`** | Near miss reported | category |
| **`fact_bulletin_ack`** | Ack event | bulletin_id, on_time_flag |
| **`fact_safety_snapshot_day`** | Daily snapshot per project | open_ca_count, overdue_ca_count, open_incidents *(gauge)* |

### 5.3 Equipment Facts

| Table | Grain | Measures |
| --- | --- | --- |
| **`fact_inspection`** | Inspection completed/overdue marked | kind (preuse/periodic), result, asset_id, on_time |
| **`fact_readiness_changed`** | Readiness transition | from/to state, reason_class |
| **`fact_deficiency`** | Opened/cleared/deferred | severity, status |
| **`fact_certification`** | Cert issued/expiring/expired | expires_at, days_to_expiry_at_event |
| **`fact_binder_completeness`** | Completeness changed | complete_flag, score |
| **`fact_maintenance_order`** | Maint lifecycle | status, overdue_flag |
| **`fact_fleet_snapshot_day`** | Daily per project/tenant | ready_count, blocked_count, oos_count, overdue_periodic |

### 5.4 Worker Facts

| Table | Grain | Measures |
| --- | --- | --- |
| **`fact_membership`** | Membership granted/updated/revoked | project_id, roles |
| **`fact_attendance`** | Attendance recorded (day) | status (present/absent/…); void corrections as compensating rows |
| **`fact_fit_signal`** | Fit signal changed | signal class *(no clinical detail)* |
| **`fact_signature_slot`** | Slot pending/completed/expired | pending_flag |
| **`fact_workforce_snapshot_day`** | Daily | active_workers, members_on_project |

### 5.5 Training Facts

| Table | Grain | Measures |
| --- | --- | --- |
| **`fact_training_assignment`** | Assignment created/completed/overdue | course_id, person_id, status |
| **`fact_training_completion`** | Completion recorded/expired/renewed | valid_from/to, expired_flag |
| **`fact_competency_gap`** | Gap opened/closed | requirement_id, gap_type |
| **`fact_orientation`** | Orientation due/completed | project_id |
| **`fact_training_snapshot_day`** | Daily | currency numerator/denom, overdue_assignments, expiring_30d |

### 5.6 COR Facts

| Table | Grain | Measures |
| --- | --- | --- |
| **`fact_cor_readiness`** | Readiness recalculated | score, covered, applicable, framework_id |
| **`fact_cor_gap`** | Gap opened/closed/overdue | element_id, severity, status |
| **`fact_cor_package`** | Evidence package generated/failed | size_bytes?, status |
| **`fact_cor_engagement`** | Engagement started/closed | type (internal/external), score_final |

### 5.7 Project / Compliance Facts

| Table | Grain | Measures |
| --- | --- | --- |
| **`fact_project_lifecycle`** | Created/activated/archived | status |
| **`fact_proof_health`** | Proof health changed | score, open_exceptions |
| **`fact_document_ack`** | Document acknowledged | document_version_id, on_time |
| **`fact_compliance_snapshot_day`** | Daily per project | composite inputs (safety/training/equipment/COR components) |

### 5.8 Platform / Ops Facts (Optional)

| Table | Use |
| --- | --- |
| **`fact_export_job`** | Export telemetry |
| **`fact_ingest_error`** | Poison messages / transform failures |
| **`fact_session`** | Security analytics (optional, high retention caution) |

---

## 6. Rollups & Materialized Aggregates

| Aggregate | Grain | Feeds |
| --- | --- | --- |
| **`agg_safety_project_day`** | tenant, project, day | FLHA rates, CA gauges, incidents |
| **`agg_equipment_project_day`** | tenant, project, day | Ready rate, overdue periodic |
| **`agg_training_project_day`** | tenant, project, day | Currency, gaps |
| **`agg_cor_subject_day`** | tenant, subject, framework, day | Readiness score, gaps |
| **`agg_project_day`** | tenant, project, day | Proof health, composite index components |
| **`agg_executive_tenant_day`** | tenant, day | Portfolio rollups |
| **`agg_*_month`** | Monthly rollup from daily | Executive historical |

Materialized views or scheduled transform jobs populate rollups from facts. Dashboards prefer rollups; custom reports may scan facts with strict limits.

---

## 7. KPI Mapping (Warehouse View)

Metrics are defined in the Analytics catalog; warehouse supplies measures. Representative mapping:

### 7.1 Safety

| KPI | Primary source |
| --- | --- |
| `safety.activities.completed` | Count `fact_safety_activity` where sealed/closed |
| `safety.flha.completion_rate` | Completed FLHA / due (due from snapshot or assignment facts) |
| `safety.toolbox.seal_rate` | Sealed toolbox / held |
| `safety.ca.open` / `overdue` | Latest `fact_safety_snapshot_day` or state from CA facts |
| `safety.ca.aging_days` | From open CA state |
| `safety.incident.count` | `fact_incident` opens |
| `safety.near_miss.count` | `fact_near_miss` |
| `safety.bulletin.ack_rate` | Ack facts / required |

### 7.2 Equipment

| KPI | Source |
| --- | --- |
| `equipment.fleet.ready_rate` | `fact_fleet_snapshot_day` |
| `equipment.preuse.compliance_rate` | `fact_inspection` preuse |
| `equipment.periodic.overdue` | Snapshot / overdue events |
| `equipment.deficiency.open` | Deficiency state |
| `equipment.cert.expiring_30d` | Cert facts / snapshot |
| `equipment.binder.complete_rate` | Binder facts |
| `equipment.oos.count` | Fleet snapshot |

### 7.3 Workers

| KPI | Source |
| --- | --- |
| `worker.active_count` | Workforce snapshot |
| `worker.assignment.coverage` | Membership + training gap join (rollup) |
| `worker.attendance.present_rate` | Attendance facts |
| `worker.signature.pending` | Signature slot facts |

### 7.4 Training

| KPI | Source |
| --- | --- |
| `training.currency_rate` | Training snapshot num/den |
| `training.assignments.overdue` | Assignment facts / snapshot |
| `training.completions.count` | Completion facts |
| `training.expiring_30d` / `expired_gaps` | Completion + gap facts |
| `training.orientation.completion_rate` | Orientation facts |
| `training.renewal.conversion_rate` | Completion renewals |

### 7.5 COR

| KPI | Source |
| --- | --- |
| `cor.readiness.score` | Latest `fact_cor_readiness` |
| `cor.elements.covered_rate` | covered/applicable |
| `cor.gaps.open` / `overdue` | Gap facts / snapshot |
| `cor.packages.generated` | Package facts |
| `cor.engagements.closed` | Engagement facts |
| `cor.score.delta_yoy` | Compare engagement scores YoY |

### 7.6 Projects & Executive

| KPI | Source |
| --- | --- |
| `project.proof_health` | `fact_proof_health` / `agg_project_day` |
| `project.open_exceptions` | Proof health payload |
| `project.compliance_completion_rate` | Composite from daily compliance snapshot |
| `compliance.composite_index` | Weighted config over domain components |
| `compliance.critical_alerts` | Thresholded open conditions |
| `exec.sites_at_risk` | Projects below configured thresholds |

Targets/thresholds live in Postgres dashboard config—not in CH ingest.

---

## 8. Domain Marts (Logical)

| Mart | Fact focus | Primary personas |
| --- | --- | --- |
| **Safety mart** | Activities, CA, incidents, bulletins | Safety coordinator |
| **Equipment mart** | Inspections, readiness, certs, binders | Equipment manager |
| **Workforce mart** | Membership, attendance, signatures pending | Supervisors / HR-lite |
| **Training mart** | Assignments, completions, gaps | Training admin |
| **COR mart** | Readiness, gaps, packages, engagements | COR admin / auditor |
| **Project mart** | Proof health, compliance daily | PM |
| **Executive mart** | Tenant/project daily rollups | Exec / sponsor |

Marts are logical views or query templates over shared dims—not isolated silos.

---

## 9. Executive Dashboards

| Dashboard | Widgets (examples) | Grain |
| --- | --- | --- |
| **Executive Scorecard** | Composite index, sites at risk, critical alerts, sparklines | Tenant / portfolio |
| **Portfolio Map/List** | Projects by proof health & COR score | Project |
| **Trend Board** | 13-month safety/training/equipment/COR | Month |
| **Exception Radar** | Top overdue CA, overdue periodic, training gaps | Project |

Freshness target: ≤ 1 hour (from Analytics Domain SLO). Always show **as-of** timestamp.

Field workers do **not** live here—Command Center / My Actions remain operational UX.

---

## 10. Historical Reports

| Report class | Behavior |
| --- | --- |
| **Standard historical** | Prebuilt: monthly safety summary, fleet compliance, training currency, COR readiness history |
| **Custom report** | Catalog metrics + dimensions + filters + date range |
| **Audit evidence assist** | Trends + deep links to `subject_id` in OLTP (not CH as evidence SoR) |
| **YoY / MoM** | Prefer `agg_*_month`; align calendars via `dim_date` |

Query guardrails: max date span, max rows, mandatory `tenant_id`, project scope intersection with AuthZ allowlist.

---

## 11. Data Refresh

### 11.1 Streaming Path (Primary)

```text
Domain commit → outbox → NATS
  → analytics-worker consumes (queue group)
  → validate schema / drop forbidden fields
  → INSERT facts (idempotent fact_id)
  → dims upsert
  → update ingest checkpoint (Postgres or CH)
```

### 11.2 Rollup Refresh

| Mechanism | Use |
| --- | --- |
| **Materialized views** | Near-real-time daily aggs |
| **Scheduled merge/transform** | Nightly month rollups, snapshot gauges |
| **Temporal rebuild** | Repair tenant/type from event archive or OLTP snapshot |

### 11.3 Snapshot Jobs

Daily (tenant-local midnight or UTC window): compute gauge snapshots (`open_ca`, fleet counts, training currency) from OLTP **read APIs** or derived state tables—document which source is authoritative for gauges when events alone are insufficient.

### 11.4 Freshness SLOs

| Class | Target |
| --- | --- |
| Operational compliance tiles | 5–15 minutes |
| Executive trends | ≤ 1 hour |
| Custom heavy reports | On-demand; show as-of |
| Full rebuild | Announced maintenance window |

### 11.5 Late & Duplicate Events

- Idempotent `fact_id` = event `event_id` when 1:1.  
- ReplacingMergeTree collapses duplicates.  
- Corrections: new compensating fact or explicit `is_correction` rows—never silent rewrite of sealed history without audit note in ops.

### 11.6 Failure Handling

- Poison → `fact_ingest_error` + alert; do not block partition.  
- Replay from NATS/archive by checkpoint.  
- CH outage: queue in worker buffer/NATS retention; dashboards show stale as-of.

---

## 12. Partitioning, Ordering, Performance

| Concern | Guidance |
| --- | --- |
| **Partition** | By `toYYYYMM(occurred_at)` or `toYYYYMMDD` for hot facts; always include `tenant_id` in ORDER BY leading key |
| **ORDER BY** | `(tenant_id, project_id, occurred_at, fact_id)` typical for project-scoped facts |
| **Skip indexes** | `event_type`, `activity_type`, `asset_id` as needed |
| **Dictionaries** | Hot dim_project / dim_metric |
| **Query API** | Only known metric SQL templates or bounded builder—no end-user raw SQL |

---

## 13. Retention

| Layer | Retention (baseline; tenant policy may extend) |
| --- | --- |
| **Raw facts** | 24–84 months operational analytics (configurable by sensitivity) |
| **Daily rollups** | 5–10 years for compliance trends |
| **Monthly rollups** | 10+ years / contractual |
| **Staging** | Days |
| **Ingest errors** | 90 days |
| **Person-level detail facts** | Shorter or Restricted access; aggregate retained longer |
| **Security session facts** | Short; optional |

TTL on MergeTree partitions for raw facts; rollups retained longer. Legal hold: suppress TTL for tenant via ops flag; exports still AuthZ-gated.

OLTP evidence retention is **separate**—CH TTL never deletes Postgres sealed records.

---

## 14. Exports

### 14.1 Flow

```text
User/API → ExportJob (Postgres)
  → AuthZ + sensitivity check
  → Temporal ExportReportWorkflow
  → Analytics query (bounded)
  → Go worker render CSV/XLSX/PDF
  → R2 FileObject
  → Notify requester
  → Audit export
```

### 14.2 Formats & Limits

| Format | Use |
| --- | --- |
| CSV / XLSX | Tabular custom + standard reports |
| PDF | Executive scorecard snapshots |

Row/byte caps; async only above threshold. PII columns omitted unless Restricted permission.

### 14.3 Scheduled Delivery

`AnalyticsSubscription` → digest/report via Notifications (email/Teams); artifact link time-boxed.

---

## 15. Security & Governance

| Control | Design |
| --- | --- |
| **Access** | Only Analytics API / workers; no BI tool direct prod access without SCD/proxy |
| **Row filters** | Inject `tenant_id` + project allowlist |
| **Column filters** | Metric sensitivity class |
| **Masking** | Hash/omit person display in standard exports |
| **Audit** | Export jobs, NL analytics queries, rebuilds |
| **Secrets** | CH credentials in secret store |

---

## 16. Relationship to Search & OLTP

| Need | System |
| --- | --- |
| Find entity by name/code | [Search Architecture](./SEARCH_ARCHITECTURE.md) |
| Enforce readiness/competency | Module APIs |
| Trend / KPI / portfolio | This warehouse |
| Evidence package bytes | COR + R2 + OLTP |

Dashboards deep-link to Place/My Actions using `subject_id`—users verify proof in SoR.

---

## 17. Quality & Observability

| Signal | Purpose |
| --- | --- |
| Ingest lag (event → CH) | Freshness SLO |
| Row counts / day vs event volume | Completeness |
| Duplicate `fact_id` rate | Idempotency health |
| Snapshot vs event-derived drift | Gauge accuracy |
| Query p95 by dashboard | Performance |
| Export failure rate | Reliability |

Data tests: not-null `tenant_id`, referential dim coverage, KPI range checks (rates 0–1).

---

## 18. Build Priority

| Phase | Deliverable |
| --- | --- |
| **P0** | Conformed dims; safety/equipment/training/project daily facts + rollups; executive scorecard |
| **P1** | COR mart; worker snapshots; custom reports; exports |
| **P2** | Long-history month rollups; advanced composite index; anomaly alerts |
| **P3** | Optional session/security mart; advanced forecasting (not SoR) |

---

## 19. Success Criteria

1. Every dashboard KPI resolves to documented facts/rollups.  
2. Ingest is idempotent and replayable from events.  
3. AuthZ-scoped queries never leak cross-project/tenant data.  
4. Freshness SLOs are measurable and visible (as-of).  
5. Retention separates raw vs rollup vs OLTP evidence.  
6. Exports are audited, capped, and PII-aware.  
7. Warehouse outage does not block field compliance operations.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Data Warehouse Architecture | ClickHouse dims, facts, KPIs, refresh, retention, exports |

---

*End of Data Warehouse & Analytics Architecture*
