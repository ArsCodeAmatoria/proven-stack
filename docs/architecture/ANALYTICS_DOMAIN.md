# Proven — Analytics Domain Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Analytics Domain Architecture (DDD) |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Principal Architecture |
| **Audience** | Engineering, Product, Design, Executive / Safety Leadership |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Domain Model](./DOMAIN_MODEL.md), [Core Domain](./CORE_DOMAIN.md), [COR Domain](./COR_DOMAIN.md), [Safety Domain](./SAFETY_DOMAIN.md), [Equipment Domain](./EQUIPMENT_DOMAIN.md), [Training Domain](./TRAINING_DOMAIN.md), [System Architecture](./SYSTEM_ARCHITECTURE.md), [UX Architecture](../ux/UX_ARCHITECTURE.md), [PRD](../PRD.md) |

---

## 1. Purpose

This document defines the **Analytics** bounded context for Proven.

Analytics is a **generic / platform subdomain** of the Construction Compliance Operating System. It provides read-optimized **compliance dashboards**, KPI suites (Safety, Equipment, Worker, Training, Project, COR readiness), **executive dashboards**, **custom reports**, and **historical trends**—powered primarily by **ClickHouse**, fed from domain events, without becoming a system of record for operational compliance entities.

**Architecture only — no implementation.**

---

## 2. Bounded Context

### 2.1 Context Card

| Field | Value |
| --- | --- |
| **Name** | Analytics & Insights |
| **Module** | `analytics` |
| **Strategic type** | Generic / platform subdomain |
| **Product metaphor** | Insight = trusted rollup of audited operational events |
| **System of record for** | Report definitions, dashboard configurations, scheduled subscriptions, metric catalog metadata, analytics ACL bindings (presentation), export job metadata |
| **Not system of record for** | Safety activities, training completions, equipment assets, COR mappings, assignments—those remain in owning modules; ClickHouse holds **analytical projections**, not authoritative writes |

### 2.2 Context Map

```text
Safety · Equipment · Training · People · Projects · Documents
Signatures · COR · Notifications · Core
        │ domain events (NATS) + occasional snapshots
        ▼
Go Analytics Workers (transform / load)
        │
        ▼
┌────────────────────────────────────────────┐
│           ClickHouse (facts + rollups)     │
└──────────────────┬─────────────────────────┘
                   │ query
┌──────────────────▼─────────────────────────┐
│               ANALYTICS                    │
│  Catalog · Dashboards · Reports · Export   │
│  (Postgres config + API + AuthZ via Core)  │
└──────────────────┬─────────────────────────┘
                   │
        Web Executive / PM / Safety dashboards
        Notifications (scheduled report delivery)
```

### 2.3 Ubiquitous Language

| Term | Meaning |
| --- | --- |
| **Metric** | Named measurable (count, rate, score, duration) |
| **Dimension** | Slice attribute (project, trade, company, region, time…) |
| **Fact Event** | Normalized analytics event derived from a domain event |
| **Aggregation** | Rollup over time buckets and dimensions |
| **Dashboard** | Curated layout of widgets/charts for a persona |
| **Custom Report** | User/tenant-defined metric + filter + visualization/export |
| **Trend** | Metric values over historical time buckets |
| **Scorecard** | Small set of executive KPIs with targets/thresholds |
| **Freshness** | Lag between source event time and analytics availability |

### 2.4 Dual-Store Principle

| Store | Role |
| --- | --- |
| **PostgreSQL (`analytics` schema)** | Report/dashboard definitions, schedules, export jobs, metric catalog |
| **ClickHouse** | High-volume facts, rollups, trend queries |
| **Owning module Postgres** | Authoritative operational truth & enforcement APIs |

Command Center / My Actions remain operational UX—not replaced by Analytics tiles ([UX Architecture](../ux/UX_ARCHITECTURE.md)).

---

## 3. Responsibilities (Mapped to Ownership)

| Responsibility | Analytics owns? | Clarification |
| --- | --- | --- |
| **Compliance Dashboards** | Yes (presentation + queries) | Data from cross-module facts |
| **Safety KPIs** | Yes (metrics derived from Safety events) | Safety remains SoR |
| **Equipment KPIs** | Yes | Equipment remains SoR / readiness API for enforcement |
| **Worker KPIs** | Yes | Aggregated person-scoped metrics; PHI minimized |
| **Training KPIs** | Yes | Training competency APIs remain enforcement |
| **Project KPIs** | Yes | Complements Projects dashboard projections |
| **COR Readiness** | Yes (trends/portfolio) | COR module owns readiness SoR & packages |
| **Executive Dashboards** | Yes | Scorecards + trends |
| **Custom Reports** | Yes | Definitions + run/export |
| **Historical Trends** | Yes | ClickHouse time series |
| **ClickHouse Integration** | Yes (pipeline contract + query layer) | Workers perform ingest I/O |

---

## 4. Aggregate Roots (Config Side)

| Aggregate | Responsibility |
| --- | --- |
| **MetricDefinition** | Catalog entry: key, type, unit, owner domain, description, sensitivity |
| **DashboardDefinition** | Persona layout, widgets, default filters |
| **ReportDefinition** | Custom/saved report config |
| **AnalyticsSubscription** | Schedule + recipients for report delivery |
| **ExportJob** | Async export request lifecycle |
| **IngestCheckpoint** | Pipeline watermark / rebuild cursor (platform ops) |

Analytical facts in ClickHouse are **not** DDD write aggregates; they are projections.

---

## 5. Entities

| Entity | Parent | Description |
| --- | --- | --- |
| **DashboardWidget** | DashboardDefinition | Chart/KPI tile config (metric, viz, dimensions) |
| **ReportColumn** | ReportDefinition | Selected metrics/dimensions |
| **ReportFilter** | ReportDefinition | FilterSpec |
| **ReportSchedule** | AnalyticsSubscription | Cron/cadence |
| **WidgetThreshold** | DashboardWidget | Warning/critical bands |
| **ExportArtifactRef** | ExportJob | FileObjectId of CSV/XLSX/PDF |
| **CatalogTag** | MetricDefinition | Grouping (safety, executive, …) |

---

## 6. Value Objects

- `MetricKey`, `MetricType` — Counter | Gauge | Rate | Ratio | Score | Duration
- `AggregationFn` — Sum | Count | CountDistinct | Avg | Min | Max | P95 | Last
- `TimeBucket` — Hour | Day | Week | Month | Quarter | Year
- `DimensionKey` — Tenant | Region | Company | Project | Area | Trade | WorkforceRole | AssetClass | Framework | Channel | …
- `FilterSpec`, `DateRange`
- `Score`, `TargetValue`, `ThresholdBand`
- `VizType` — KPI | Line | Bar | StackedBar | Heatmap | Table | Scorecard | Pie*(discouraged for many categories)*
- `FreshnessSLO`, `SensitivityClass` — Standard | Restricted | PII
- `DashboardPersona` — Executive | Safety | PM | Equipment | Training | COR
- `ExportFormat` — CSV | XLSX | PDF

---

## 7. Metrics Catalog (Representative)

### 7.1 Safety KPIs

| MetricKey | Description | Agg |
| --- | --- | --- |
| `safety.activities.completed` | Closed/sealed activities | Count |
| `safety.flha.completion_rate` | Required FLHA completed / due | Rate |
| `safety.toolbox.seal_rate` | Toolbox talks fully sealed | Rate |
| `safety.ca.open` | Open corrective actions | Gauge |
| `safety.ca.overdue` | Overdue CAs | Gauge |
| `safety.ca.aging_days` | Age of open CAs | Avg/P95 |
| `safety.incident.count` | Incidents opened | Count |
| `safety.near_miss.count` | Near misses reported | Count |
| `safety.risk.critical_share` | Share of Critical residual risk submits | Rate |
| `safety.bulletin.ack_rate` | Bulletin acknowledgement completion | Rate |

### 7.2 Equipment KPIs

| MetricKey | Description |
| --- | --- |
| `equipment.fleet.ready_rate` | Assets Ready / in-scope assets |
| `equipment.preuse.compliance_rate` | Pre-use completed in validity window |
| `equipment.periodic.overdue` | Overdue periodic inspections |
| `equipment.deficiency.open` | Open deficiencies |
| `equipment.cert.expiring_30d` | Certs expiring in 30 days |
| `equipment.binder.complete_rate` | Tower/self-erect binders complete |
| `equipment.oos.count` | Out-of-service assets |

### 7.3 Worker KPIs

| MetricKey | Description |
| --- | --- |
| `worker.active_count` | Active people in scope |
| `worker.assignment.coverage` | Assigned workers with no critical gaps |
| `worker.eligibility.ready_rate` | Workers Ready by composed signal snapshot *(analytical—not enforcement)* |
| `worker.attendance.present_rate` | Workforce attendance present rate |
| `worker.signature.pending` | Pending signature slots (from Signatures events) |

Minimize PII; prefer counts/rates over individual lists in executive views. Person-level drill-down requires stricter AuthZ.

### 7.4 Training KPIs

| MetricKey | Description |
| --- | --- |
| `training.currency_rate` | Valid competencies / required |
| `training.assignments.overdue` | Overdue assignments |
| `training.completions.count` | Completions recorded |
| `training.expiring_30d` | Completions expiring in 30 days |
| `training.expired_gaps` | Expired/missing cells |
| `training.orientation.completion_rate` | Site orientation complete |
| `training.renewal.conversion_rate` | Expiring → renewed |

### 7.5 Project KPIs

| MetricKey | Description |
| --- | --- |
| `project.proof_health` | Proof health score trend |
| `project.open_exceptions` | Open actionable exceptions |
| `project.participant.count` | Active company participants |
| `project.membership.count` | Active worker memberships |
| `project.compliance_completion_rate` | Composite completion across required controls |

### 7.6 COR Readiness

| MetricKey | Description |
| --- | --- |
| `cor.readiness.score` | Overall readiness score |
| `cor.elements.covered_rate` | Covered / applicable elements |
| `cor.gaps.open` | Open gaps |
| `cor.gaps.overdue` | Overdue gaps |
| `cor.packages.generated` | Evidence packages generated |
| `cor.engagements.closed` | Closed audits |
| `cor.score.delta_yoy` | vs prior historical audit |

### 7.7 Compliance / Executive Rollups

| MetricKey | Description |
| --- | --- |
| `compliance.composite_index` | Weighted index (config) across safety/training/equipment/COR |
| `compliance.critical_alerts` | Count of critical open conditions |
| `exec.sites_at_risk` | Projects below proof/COR thresholds |

Targets/thresholds live in DashboardWidget config—not hard-coded in ingest.

---

## 8. Dimensions

| DimensionKey | Source | Used by |
| --- | --- | --- |
| `tenant_id` | All events | Isolation |
| `region_code` | Tenant/project | Executive, COR |
| `company_id` | Participants / employment | GC/Sub views |
| `project_id` | Most operational events | Project KPIs |
| `area_id` | Safety/equipment optional | Site heatmaps |
| `trade_code` | People | Worker/training |
| `workforce_role` | People | Worker KPIs |
| `asset_class` / `asset_type_id` | Equipment | Equipment KPIs |
| `activity_type` | Safety | Safety KPIs |
| `framework_id` / `framework_version` | COR | COR KPIs |
| `channel` | Notifications (optional) | Comms analytics |
| `time` (`event_date`, bucket) | Everywhere | Trends |

Dimension members resolved via ACL-safe snapshots in facts (ids + optional display names refreshed asynchronously).

---

## 9. Events (Analytics Fact Model)

### 9.1 Inbound Domain Events (Examples)

Consumed via NATS → workers → ClickHouse:

- Safety: `SafetyActivityClosed`, `CorrectiveAction*`, `Incident*`, `SafetyBulletinAcknowledged`, …
- Equipment: `Inspection*`, `AssetReadinessChanged`, `Certification*`, `Deficiency*`, `BinderCompletenessChanged`, …
- Training: `TrainingCompletion*`, `TrainingAssignmentOverdue`, `CompetencyGap*`, `Renewal*`, …
- Projects: `Project*`, `ProjectProofHealthChanged`, …
- People: `Person*`, `WorkforceRole*`, `Attendance*` (aggregated carefully)
- Documents: `DocumentAcknowledged`, `DocumentVersionPublished`, …
- Signatures: `SignaturePackageCompleted`, `SignaturePackageExpired`, …
- COR: `ReadinessRecalculated`, `Gap*`, `EvidencePackageGenerated`, `AuditEngagementClosed`, …
- Notifications: optional delivery metrics events

### 9.2 Normalized Fact Event Envelope

Logical ClickHouse row / JSON fact:

- `fact_id`, `event_type`, `event_version`
- `occurred_at`, `ingested_at`
- `tenant_id`, dimension ids…
- `metric_keys` touched / measures payload
- `subject_ref` (module, type, id)—no foreign blobs
- `correlation_id`

### 9.3 Analytics Module Domain Events (Config)

- `MetricDefinitionPublished`
- `DashboardDefinitionPublished`
- `ReportDefinitionPublished`
- `AnalyticsSubscriptionChanged`
- `ExportJobCompleted` / `Failed`
- `AnalyticsProjectionRebuilt`
- `ScheduledReportDispatched`

---

## 10. Aggregations

### 10.1 Layers

| Layer | Where | Purpose |
| --- | --- | --- |
| **Raw facts** | ClickHouse merge tree | Replayable grain |
| **Daily rollups** | ClickHouse sum/agg tables / materialized views | Dashboard speed |
| **Monthly rollups** | ClickHouse | Executive trends |
| **On-query agg** | ClickHouse SQL | Custom reports ad hoc |

### 10.2 Rules

1. Prefer incremental ingest from events; periodic rebuild from archive for repair.  
2. Late events: idempotent inserts / ReplacingMergeTree strategies as appropriate.  
3. Do not aggregate away audit provenance needed for drill-down links (keep subject refs at fact grain).  
4. Enforcement paths never wait on aggregations.

### 10.3 Freshness SLO (Initial Targets)

| Dashboard class | Freshness target |
| --- | --- |
| Operational compliance tiles | ≤ 5–15 minutes |
| Executive trends | ≤ 1 hour |
| Custom heavy reports | On-demand; show as-of timestamp |

---

## 11. Charts & Dashboards

### 11.1 Visualization Mapping

| Need | VizType |
| --- | --- |
| Single KPI | KPI / Scorecard |
| Trend over time | Line |
| Compare projects/trades | Bar / StackedBar |
| Element/project heat | Heatmap |
| Tabular export-oriented | Table |

Avoid chart junk; align with UX calm hierarchy—Analytics is not the Home for field workers.

### 11.2 Dashboard Sets

| Dashboard | Persona | Primary metrics |
| --- | --- | --- |
| **Compliance Overview** | Safety lead / PM | Composite index, exceptions, proof health |
| **Safety Performance** | Safety | FLHA/toolbox rates, CA aging, incidents |
| **Equipment Fleet** | Equipment mgr | Ready rate, overdue periodic, binder, certs |
| **Workforce Competency** | Training / supervisors | Currency, gaps, orientation |
| **Project Portfolio** | PM / exec | Per-project proof, exceptions |
| **COR Readiness** | Audit sponsors | Score, coverage, gaps, packages |
| **Executive Scorecard** | Executive | Few KPIs + trend sparklines + sites at risk |

### 11.3 Widget Config

Each widget declares: `MetricKey`, `AggregationFn`, `TimeBucket`, `Dimensions`, `FilterSpec`, `VizType`, thresholds, drill-down route (deep link to module lists—not Analytics as SoR).

---

## 12. Custom Reports

1. Users compose metrics + dimensions + filters + date range from catalog (permission-filtered).  
2. Save as `ReportDefinition`.  
3. Run sync for small queries; async `ExportJob` for large.  
4. Subscriptions email/Teams via Notifications (artifact link).  
5. Custom reports cannot invent metrics not in catalog without platform publish.

---

## 13. Historical Trends

1. Retain raw/rollup history per tenant retention policy (analytics retention class).  
2. Support compare ranges (WoW, MoM, YoY) especially for COR and safety rates.  
3. Framework version changes must not silently rewrite past COR scores—tag `framework_version` dimension.  
4. Rebuild jobs produce `AnalyticsProjectionRebuilt` for ops visibility.

---

## 14. ClickHouse Integration

### 14.1 Pipeline

```text
Domain outbox → NATS
  → Go analytics-worker
  → validate/normalize fact
  → insert ClickHouse (batch)
  → update ingest checkpoint
```

### 14.2 Responsibilities Split

| Component | Responsibility |
| --- | --- |
| Owning modules | Emit correct domain events |
| Go workers | Transform/load I/O; retries; **no KPI business reinvention** |
| Analytics API (Rust) | AuthZ, catalog, dashboards, parameterized CH queries |
| ClickHouse | Store/aggregate |

### 14.3 Query Safety

1. Every query injects `tenant_id` predicate from auth context.  
2. Project/company scopes applied from Core grants/membership.  
3. No end-user free-form SQL.  
4. Query templates bound to MetricDefinition allowlists.  
5. Row limits + timeouts; spill large results to export jobs.

### 14.4 Failure Modes

| Failure | Behavior |
| --- | --- |
| CH outage | Dashboards show stale/unavailable; OLTP unaffected |
| Ingest lag | Surface freshness warning |
| Poison event | DLQ; skip metric updates; alert ops |
| Rebuild | Backfill from event archive without locking OLTP |

---

## 15. Permissions

| Code | Intent |
| --- | --- |
| `analytics.dashboard.read` | View permitted dashboards |
| `analytics.dashboard.manage` | Configure dashboards (admin) |
| `analytics.report.read` | Run/view reports |
| `analytics.report.manage` | Create/edit custom reports |
| `analytics.subscription.manage` | Schedules |
| `analytics.export.create` | Start exports |
| `analytics.catalog.read` | Browse metric catalog |
| `analytics.catalog.manage` | Publish metrics (platform) |
| `analytics.worker.kpi.read` | Person-level worker drill-down |
| `analytics.exec.read` | Executive scorecard |
| `analytics.cor.read` | COR analytics views |
| `analytics.admin.rebuild` | Trigger rebuilds |

Persona access is RBAC + scope. Field workers typically have **no** portfolio analytics by default.

---

## 16. Export

### 16.1 Flow

```text
Create ExportJob(report/dashboard slice, format)
  → authorize
  → async query ClickHouse
  → write artifact via Core Files
  → notify requester
  → download via authorized URL
```

### 16.2 Rules

1. Exports inherit the same AuthZ filters as interactive queries.  
2. PII/restricted metrics require elevated permission and are watermarked/audited.  
3. Export artifacts follow retention settings.  
4. Core Audit on export create/download for sensitive classes.  
5. Formats: CSV/XLSX primary; PDF for scorecard snapshots optional.

---

## 17. Relationships (Logical)

```text
MetricDefinition 1──* DashboardWidget
ReportDefinition ──uses──► MetricDefinition*
AnalyticsSubscription ──runs──► ReportDefinition ──► ExportJob ──► Notifications
Fact events (CH) ──queried by──► Dashboards/Reports
COR/Safety/… modules ──events──► facts (no reverse write)
```

---

## 18. Business Rules / Anti-Corruption

1. Analytics **never** writes Safety/Training/Equipment/COR business state.  
2. Insights may deep-link to My Actions / module lists; starting workflows requires explicit user/command in owning module.  
3. “Eligibility ready rate” in Analytics is descriptive snapshot—not `GetPersonCompetency` substitute.  
4. Equipment readiness enforcement remains `GetAssetReadiness`.  
5. COR package generation remains COR module.  
6. Prefer event truth over scraping OLTP tables.  
7. Redis may cache dashboard JSON briefly—not factual SoR.

---

## 19. Reporting vs Module Reports

| Kind | Owner |
| --- | --- |
| Operational lists (open CAs, overdue inspections) | Owning modules |
| COR element evidence index | COR |
| Portfolio trends, executive scorecards, custom CH reports | **Analytics** |

Duplicate metrics may appear in both places initially; Analytics is the portfolio/history system of insight.

---

## 20. Audit Trail

Core-audit:

- Dashboard/report definition publish  
- Subscription changes  
- Export create/download (restricted)  
- Rebuild triggers  
- Catalog metric publish  

---

## 21. Data Ownership Summary

| Data | Owner |
| --- | --- |
| Dashboard/report config | Analytics Postgres |
| Facts/rollups | ClickHouse (analytics-managed pipeline) |
| File exports | Core Files |
| Source events | Producing modules |
| AuthZ | Core |

---

## 22. Success Criteria

Analytics is correctly designed when:

1. Executives and safety leaders see calm, trustworthy scorecards—not chart spam on Home.  
2. Safety/Equipment/Training/Project/COR KPIs share one metric/dimension model.  
3. ClickHouse absorbs trend volume without slowing OLTP.  
4. Custom reports stay within catalog + AuthZ guardrails.  
5. Freshness is visible; outages degrade insights only.  
6. Analytics never becomes a shadow system of record or enforcement engine.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Principal Architecture | Initial Analytics domain architecture (ClickHouse-backed) |

---

*End of Analytics Domain Architecture*
