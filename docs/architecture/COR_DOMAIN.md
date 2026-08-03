# Proven — COR Domain Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | COR Domain Architecture (DDD) |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Principal Architecture |
| **Audience** | Engineering, Product, Design, Safety / Audit Leadership |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Domain Model](./DOMAIN_MODEL.md), [Core Domain](./CORE_DOMAIN.md), [Projects Domain](./PROJECTS_DOMAIN.md), [Safety Domain](./SAFETY_DOMAIN.md), [Training Domain](./TRAINING_DOMAIN.md), [Documents Domain](./DOCUMENTS_DOMAIN.md), [Equipment Domain](./EQUIPMENT_DOMAIN.md), [System Architecture](./SYSTEM_ARCHITECTURE.md), [PRD](../PRD.md) |

---

## 1. Purpose

This document defines the **COR** bounded context for Proven.

COR is a **strategic core domain** of the Construction Compliance Operating System. It organizes continuous audit readiness and formal audit execution against **BCCSA COR**, **SECOR**, and **future regional compliance standards**—planning audits, managing elements, interviews, evidence, observations, findings/corrective actions, scoring, reports, internal audits, external preparation, history, dashboards, and analytics.

COR is a **consumer and organizer of proof**, not a second system of record for Safety, Training, Documents, or Equipment data.

**Documentation only — no implementation.**

---

## 2. Bounded Context

### 2.1 Context Card

| Field | Value |
| --- | --- |
| **Name** | COR Audit Readiness |
| **Module** | `cor` (alias `cor_audit`) |
| **Strategic type** | Core domain (differentiating) |
| **Product metaphor** | Readiness = continuous mapped evidence; Audit = time-boxed evaluation against a framework |
| **System of record for** | Compliance frameworks (COR/SECOR/…), elements, audit plans, audit engagements (internal/external prep), interviews, observations, evidence mappings & packages, audit findings, audit corrective actions (or links to Safety CAs), scoring models/results, audit reports, historical audit records, COR dashboard/analytics projections |
| **Not system of record for** | Underlying safety activities, training completions, controlled docs, equipment inspections (those modules remain SoR); signature blobs (Signatures); file bytes (Core Files); AuthZ (Core) |

### 2.2 Supported Standards (Initial + Extensible)

| Framework family | Examples | Design approach |
| --- | --- | --- |
| **BCCSA COR** | BC Certificate of Recognition / partner agency COR programs | Versioned `AuditFramework` pack |
| **SECOR** | Small Employer Certificate of Recognition | Separate framework pack (elements/scoring differ) |
| **Future regional** | Other CA provinces, AU/NZ/US analogs, client-specific | New framework packs without code forks—config + mapping |

All standards share the same COR metamodel: **Framework → Elements → Evidence mappings → Audits → Scores → Reports**.

### 2.3 Context Map

```text
Safety · Training · Documents · Equipment · Signatures · Projects · People
        │ (events + query APIs + provenance refs)
        ▼
┌──────────────────────────────────────────────┐
│                     COR                      │
│  Frameworks · Readiness · Audits · Scoring   │
│  Evidence Packages · Reports · History       │
└──────────────────┬───────────────────────────┘
                   │
        Notifications · Workflows · Analytics · Core Audit
```

### 2.4 Ubiquitous Language

| Term | Meaning |
| --- | --- |
| **Framework** | Versioned standard definition (BCCSA COR 202x, SECOR 202x, …) |
| **Element** | Auditable requirement unit within a framework |
| **Readiness Profile** | Continuous coverage state of a tenant/company/project against a framework |
| **Audit Plan** | Planned schedule and scope for audit work |
| **Audit Engagement** | Concrete internal or external-prep audit instance |
| **Interview** | Structured interview record with interviewee and notes/score inputs |
| **Observation** | Auditor observation of worksite/practice |
| **Evidence Mapping** | Link from element → provenance refs in other modules |
| **Evidence Package** | Exportable bundle for external auditors |
| **Finding** | Gap/nonconformance identified in an audit |
| **Audit Corrective Action** | Remediation tracked for a finding (may link Safety CA) |
| **Score** | Element and overall scoring per framework rules |
| **Historical Audit** | Closed engagement retained for trend/compare |

---

## 3. Responsibilities (Mapped to Ownership)

| Responsibility | COR owns? | Clarification |
| --- | --- | --- |
| **Audit Planning** | Yes | Plans, schedules, scope, assignees |
| **Elements** | Yes | Per framework version |
| **Interviews** | Yes | Within audit engagement |
| **Evidence** | Mappings + packages | Source evidence remains in producing modules |
| **Observations** | Yes | Auditor observations |
| **Corrective Actions** | Audit CA aggregate and/or links | Operational field CAs owned by Safety; COR findings may **spawn or link** Safety CAs |
| **Scoring** | Yes | Framework-specific score calculation |
| **Reports** | Yes | Audit report generation metadata + outputs |
| **Internal Audits** | Yes | Engagement type Internal |
| **External Audit Preparation** | Yes | Engagement type ExternalPrep + packages |
| **Historical Audits** | Yes | Closed engagements + immutable snapshots |
| **Dashboard** | Yes | COR readiness/audit dashboards |
| **Analytics** | Projections + events | Heavy trends may land in platform Analytics/ClickHouse |

---

## 4. Aggregate Roots

| Aggregate | Responsibility |
| --- | --- |
| **AuditFramework** | Published standard pack: metadata, region, version, scoring model ref |
| **FrameworkElement** | Element definition (or entity under framework if tightly coupled; separate aggregate if reused/customized heavily) |
| **ReadinessProfile** | Continuous coverage for a subject (tenant/company/project) + framework |
| **EvidenceMapping** | Persistent link element ↔ evidence provenance (may be entities under ReadinessProfile) |
| **AuditPlan** | Calendar/scope plan |
| **AuditEngagement** | Internal or external-prep audit run |
| **Interview** | Interview instance in an engagement |
| **Observation** | Observation instance in an engagement |
| **AuditFinding** | Finding/nonconformance |
| **AuditCorrectiveAction** | CA for audit finding (with optional Safety CA link) |
| **Scorecard** | Calculated scores for readiness or engagement |
| **EvidencePackage** | Assembled export package |
| **AuditReport** | Generated report artifact metadata |
| **CorDashboardProjection** | Dashboard read model |
| **CorAnalyticsProjection** | Analytics read model |

---

## 5. Entities

### 5.1 Framework & Elements

| Entity | Description |
| --- | --- |
| **ElementGuideline** | Auditor guidance text |
| **ElementScoreRule** | Points, weights, pass thresholds |
| **ElementEvidenceHint** | Suggested evidence types (SafetyActivity, Policy Ack, TrainingCompletion, …) |
| **FrameworkLocale** | Language/region variants |
| **ApplicabilityRule** | SECOR vs COR size/employer rules (config) |

### 5.2 Readiness & Evidence

| Entity | Description |
| --- | --- |
| **CoverageCell** | Element coverage status + confidence |
| **GapItem** | Missing/partial coverage with owner/due |
| **ProvenanceRef** | Module, aggregate type/id, version/hash, event id, occurred_at |
| **MappingNote** | Human rationale for manual mappings |

### 5.3 Planning & Engagement

| Entity | Description |
| --- | --- |
| **PlanMilestone** | Planned dates (kickoff, fieldwork, close) |
| **ScopeCompany** / **ScopeProject** | What is in scope |
| **EngagementTeamMember** | Lead auditor, assistants |
| **EngagementChecklistItem** | Prep tasks |

### 5.4 Interviews, Observations, Findings

| Entity | Description |
| --- | --- |
| **InterviewQuestion** | Question bank item / response |
| **IntervieweeRef** | PersonId + role snapshot |
| **ObservationAttachmentRef** | FileObjectId photos/notes |
| **FindingElementLink** | Elements affected |
| **FindingEvidenceRef** | Supporting/contra evidence |
| **CALink** | Link to AuditCorrectiveAction and/or Safety `CorrectiveActionId` |

### 5.5 Packages & Reports

| Entity | Description |
| --- | --- |
| **PackageItem** | Ordered evidence item with provenance |
| **PackageManifest** | Hash inventory for integrity |
| **ReportSection** | Rendered sections snapshot |
| **ReportArtifactRef** | FileObjectId of PDF/export |

### 5.6 History

| Entity | Description |
| --- | --- |
| **EngagementSnapshot** | Immutable score/element results at close |
| **ComparisonIndex** | Links prior historical audits for delta views |

---

## 6. Value Objects

- `FrameworkId`, `FrameworkVersion`, `FrameworkFamily` — BCCSA_COR | SECOR | CUSTOM_REGIONAL | …
- `ElementCode`, `ElementId`
- `CoverageStatus` — Covered | Partial | Missing | NotApplicable | Unknown
- `ReadinessScore`, `ElementScore`, `OverallScore`
- `AuditPlanId`, `AuditEngagementId`
- `EngagementType` — Internal | ExternalPreparation | Mock | Surveillance
- `EngagementStatus` — Planned | InProgress | Scoring | Reporting | Closed | Cancelled
- `FindingSeverity` — Minor | Major | Critical (framework-mapped)
- `FindingStatus` — Open | InRemediation | Verified | Closed
- `PackageStatus` — Requested | Assembling | Ready | Failed | Expired
- `ReportStatus` — Draft | Final | Archived
- `RegionCode`, `SubjectRef` (Tenant/Company/Project)
- `ProvenanceRef`, `FileObjectId`, `SignaturePackageId`
- `ScoreModelId`

---

## 7. Relationships

```text
AuditFramework 1──* FrameworkElement
        │
        ├── ReadinessProfile (per subject)
        │     ├── CoverageCell per element
        │     ├── EvidenceMapping *──► ProvenanceRef (foreign modules)
        │     └── GapItem
        │
        └── AuditPlan 1──* AuditEngagement
              ├── Interview *
              ├── Observation *
              ├── AuditFinding *──► AuditCorrectiveAction
              │         └── optional link ──► Safety.CorrectiveAction
              ├── Scorecard
              ├── EvidencePackage *
              └── AuditReport *
                    └── EngagementSnapshot (on close) ──► Historical Audits

Dashboard/Analytics projections ◄── readiness + engagement events
```

### 7.1 Evidence Relationship (Critical)

```text
COR EvidenceMapping
  → points at SafetyActivityId / TrainingCompletionId / DocumentVersionId /
    AcknowledgementId / InspectionId / CertificationRecordId / SignaturePackageId / …
  → does NOT copy mutable foreign aggregates
  → package assembly queries foreign modules via public APIs at generate-time
    and stores sealed snapshot refs + hashes in PackageItem
```

### 7.2 COR CA vs Safety CA

| Kind | Owner | Use |
| --- | --- | --- |
| Field operational CA | Safety | Day-to-day remediation |
| Audit finding CA | COR | Audit nonconformance remediation |
| Link | COR → Safety | When finding maps to operational fix already tracked |

---

## 8. Framework Pack Design (BCCSA COR, SECOR, Future)

### 8.1 Pack Contents

Each published `AuditFramework` version includes:

1. Element tree/codes and titles  
2. Guidelines and evidence hints  
3. Scoring model (weights, thresholds, NA rules)  
4. Interview question banks (optional)  
5. Applicability rules (employer size, industry—**configuration**, not hardcoded forks)  
6. Locale strings  

### 8.2 Evolution Rules

1. Frameworks are versioned; in-flight audits pin a version.  
2. New regional standards = new pack publication, same metamodel.  
3. Mapping hints evolve additively; readiness recompute jobs migrate carefully.  
4. Legal/program interpretation stays in content packs + customer config—not scattered in Safety/Training code.

---

## 9. Domain Events

### 9.1 Framework & Readiness

- `AuditFrameworkPublished`
- `AuditFrameworkRetired`
- `ReadinessProfileInitialized`
- `EvidenceLinkedToElement`
- `EvidenceUnlinked`
- `ReadinessRecalculated`
- `GapOpened` / `GapClosed` / `GapAssigned`

### 9.2 Planning & Engagements

- `AuditPlanCreated` / `Updated`
- `AuditEngagementOpened`
- `AuditEngagementStatusChanged`
- `InterviewRecorded`
- `ObservationRecorded`
- `AuditFindingOpened` / `Updated` / `Closed`
- `AuditCorrectiveActionOpened` / `Completed` / `Verified`
- `ScorecardCalculated`
- `AuditEngagementClosed`
- `HistoricalAuditRecorded`

### 9.3 Packages & Reports

- `EvidencePackageRequested`
- `EvidencePackageGenerated`
- `EvidencePackageFailed`
- `AuditReportGenerated`
- `AuditReportFinalized`

### 9.4 Dashboard / Analytics

- `CorDashboardRebuilt`
- `CorAnalyticsProjectionUpdated`

---

## 10. Business Rules

### 10.1 Readiness (Continuous)

1. Readiness updates from upstream events (conformist handlers) + manual mappings.  
2. Coverage status derives from mapping quality rules per evidence hint (e.g., requires sealed activity within window).  
3. Gaps get owners/due dates; overdue notifies.  
4. Readiness score uses framework score model—not a vanity KPI.  
5. Manual “mark covered” without provenance forbidden for audit-grade elements (or allowed only with elevated permission + note).

### 10.2 Audit Planning

1. Plan must specify framework version + subject scope.  
2. Overlapping engagements on same subject/version allowed only with explicit type (e.g., mock vs official).  
3. External prep engagements should generate at least one EvidencePackage before “ready” status.

### 10.3 Interviews & Observations

1. Interviews reference People `PersonId` when internal; guest interviewee snapshot allowed for external parties.  
2. Observations may attach photos via Core Files.  
3. Both can contribute to findings and scoring inputs.

### 10.4 Findings & Corrective Actions

1. Finding must link ≥1 element (unless framework allows general findings).  
2. Critical findings may block “audit close” until CA plan exists (configurable).  
3. Closing finding requires verified CA or formal acceptance per policy.  
4. Linking Safety CA does not transfer ownership of operational CA to COR.

### 10.5 Scoring

1. Scoring deterministic for a framework version + inputs snapshot.  
2. Recalculation audited when inputs change mid-engagement.  
3. SECOR vs COR differences live entirely in score rules packs.  
4. Final scores frozen into EngagementSnapshot on close.

### 10.6 Evidence Packages

1. Package generation is Temporal long-running; idempotent by request id.  
2. Manifest stores hashes/provenance for each item.  
3. Package stored as FileObject(s); COR keeps metadata.  
4. Packages expire/regenerate; historical packages retained with engagement.

### 10.7 Historical Audits

1. Closed engagements are immutable (corrections = amend record with new versioned note, not silent edit).  
2. History supports year-over-year element comparison.  
3. Used for dashboard trends and external prep baselining.

### 10.8 Multi-Subject

1. Readiness may be tenant-, company-, or project-scoped per customer program.  
2. BCCSA COR often company-scoped; project overlays supported for GC programs.  
3. AuthZ scopes all reads/writes accordingly.

---

## 11. Workflow Integration

| Workflow | Purpose |
| --- | --- |
| `ReadinessRecomputeWorkflow` | Batch/event-driven coverage recalculation |
| `GapEscalationWorkflow` | Overdue gaps |
| `AuditEngagementWorkflow` | Plan → fieldwork → scoring → report → close |
| `InterviewScheduleWorkflow` | Reminders for planned interviews |
| `FindingCaWorkflow` | Finding → CA → verify → close |
| `EvidencePackageWorkflow` | Gather → hash → store → notify |
| `ExternalPrepWorkflow` | Checklist for external audit readiness |
| `ReportGenerationWorkflow` | Assemble final report artifact |

### 11.1 External Preparation Sequence

```text
Open AuditEngagement(ExternalPreparation, frameworkVersion)
  → ensure ReadinessProfile fresh
  → assign gaps
  → Request EvidencePackage
  → Temporal assembles via foreign queries
  → generate AuditReport draft
  → mark prep Ready / notify sponsors
  → on external completion date, optionally close + HistoricalAuditRecorded
```

### 11.2 Package Assembly Activities

```text
For each mapped ProvenanceRef:
  → call owning module public query
  → fetch authorized file/export slice
  → write PackageItem + checksum
→ write manifest → R2 via Core Files → Package Ready
```

---

## 12. Permissions

| Code | Intent |
| --- | --- |
| `cor.framework.read` | View framework packs |
| `cor.framework.manage` | Publish/retire packs (platform/tenant admin) |
| `cor.readiness.read` | View readiness/dashboard |
| `cor.readiness.manage` | Manual mappings, gap owners |
| `cor.plan.manage` | Audit plans |
| `cor.engagement.manage` | Run internal/external prep audits |
| `cor.interview.record` | Record interviews |
| `cor.observation.record` | Record observations |
| `cor.finding.manage` | Findings |
| `cor.ca.manage` | Audit corrective actions |
| `cor.score.calculate` | Trigger scoring |
| `cor.package.generate` | Evidence packages |
| `cor.report.manage` | Reports finalize |
| `cor.history.read` | Historical audits |
| `cor.analytics.read` | COR analytics |

Scopes: Tenant/Company/Project per subject; external auditors may receive time-boxed package links—not full module admin.

---

## 13. Notifications

| Trigger | Audience |
| --- | --- |
| Gap opened/overdue | Gap owner, safety lead |
| Engagement starting | Audit team |
| Interview due | Interviewer/interviewee |
| Finding opened (esp. critical) | Sponsor, safety lead |
| Audit CA overdue | Owner + escalation |
| Evidence package ready/failed | Requester |
| Report finalized | Sponsors |
| External prep checklist incomplete | Plan owner |
| Readiness score drop beyond threshold | Leadership digest |

COR emits events; Notifications delivers.

---

## 14. Reports & Dashboard & Analytics

### 14.1 Dashboard (COR-owned UX projections)

| Block | Content |
| --- | --- |
| Overall readiness score | Against selected framework |
| Element heat map | Covered/Partial/Missing |
| Open gaps | Owners/due |
| Upcoming audits | From plans |
| Package status | External prep |
| Trend vs last historical audit | Delta |

### 14.2 Reports

| Report | Purpose |
| --- | --- |
| Readiness summary | Continuous state |
| Element evidence index | What proves each element |
| Gap register | Remediation tracking |
| Internal audit report | Formal output |
| External prep binder/package inventory | Auditor handoff |
| Finding & CA register | Close-out tracking |
| Historical comparison | Year-over-year |
| Score breakdown | Element points |

Artifacts: metadata in COR + PDF/export via Core Files; generation via workflow.

### 14.3 Analytics

| Layer | Use |
| --- | --- |
| COR projections | Operational COR analytics |
| Platform Analytics / ClickHouse | Multi-tenant/portfolio trends (events) |

Analytics never become authoritative coverage SoR.

---

## 15. Public Interfaces & API (Summary)

### 15.1 Interfaces

| Interface | Purpose |
| --- | --- |
| `CorFrameworkApi` | List/get frameworks/elements |
| `CorReadinessApi` | Coverage, gaps, scores |
| `CorAuditApi` | Plans, engagements, interviews, observations, findings |
| `CorPackageApi` | Request/status/download metadata |
| `CorReportApi` | Report generate/get |
| `CorHistoryApi` | Historical audits/compare |

### 15.2 HTTP (Illustrative)

Base: `/api/cor`

- `/frameworks`, `/frameworks/{id}/elements`
- `/readiness`, `/gaps`
- `/plans`, `/engagements`
- `/engagements/{id}/interviews`, `/observations`, `/findings`, `/scorecard`
- `/packages`, `/reports`
- `/history`, `/dashboard`, `/analytics`

AuthN/AuthZ via Core; package download authorized and audited.

---

## 16. Audit Trail

Core Audit must record:

- Framework publish/retire  
- Manual evidence map/unmap  
- Engagement open/close  
- Finding open/close  
- Score finalize  
- Package generate  
- Report finalize  
- Historical amendments  

COR snapshots provide program evidence; Core Audit provides security/accountability.

---

## 17. Data Ownership

### 17.1 Schema `cor` Owns

- Frameworks, elements, score rules  
- Readiness profiles, mappings, gaps  
- Plans, engagements, interviews, observations  
- Findings, audit CAs  
- Scorecards, packages metadata, reports metadata  
- Historical snapshots, dashboard/analytics projections  

### 17.2 Never Owned by COR

| Data | Owner |
| --- | --- |
| FLHA/incident records | Safety |
| Training completions | Training |
| Policies/SWP versions | Documents |
| Asset inspections/certs | Equipment |
| Signature evidence | Signatures |
| File bytes | Core Files |

---

## 18. Integration With Other Modules

| Module | COR consumes | COR provides |
| --- | --- | --- |
| **Safety** | Activities, CAs, incidents, bulletins events/APIs | Optional finding→Safety CA links; readiness demand signals |
| **Training** | Completions, matrix gaps | Element coverage for competency elements |
| **Documents** | Effective policies/SWP/acks | Element coverage for documentation elements |
| **Equipment** | Inspections, certs, binders | Equipment program elements |
| **Signatures** | Package/ack provenance | Proof integrity in packages |
| **Projects / People** | Scope subjects, interviewees | Scoped readiness |
| **Notifications / Workflows** | — | Fan-out + durable audit processes |
| **Analytics** | — | Events for portfolio COR trends |

---

## 19. Anti-Patterns

1. Duplicating Safety/Training rows into COR as mutable SoR  
2. Hardcoding BCCSA vs SECOR branches across the monolith instead of framework packs  
3. Scoring in spreadsheets outside Scorecard aggregate  
4. External prep without hashed evidence manifests  
5. Editing closed historical audits silently  
6. Letting dashboard tiles invent coverage without mappings  
7. Granting external auditors blanket tenant admin  

---

## 20. Success Criteria

COR is correctly designed when:

1. BCCSA COR and SECOR run as versioned packs on one metamodel.  
2. New regional standards ship as content/config, not module forks.  
3. Continuous readiness reflects live operational evidence with provenance.  
4. Internal audits and external prep produce defensible packages and reports.  
5. Findings/CAs close with clear links to operational remediation where needed.  
6. History enables comparison without rewriting the past.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Principal Architecture | Initial COR domain architecture (BCCSA COR, SECOR, extensible packs) |

---

*End of COR Domain Architecture*
