# Proven — Safety Domain Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Safety Domain Architecture (DDD) |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Principal Architecture |
| **Audience** | Engineering, Product, Design, Safety Leadership |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Domain Model](./DOMAIN_MODEL.md), [Core Domain](./CORE_DOMAIN.md), [Projects Domain](./PROJECTS_DOMAIN.md), [People Domain](./PEOPLE_DOMAIN.md), [System Architecture](./SYSTEM_ARCHITECTURE.md), [UX Architecture](../ux/UX_ARCHITECTURE.md), [PRD](../PRD.md) |

---

## 1. Purpose

This document defines the **Safety** bounded context for Proven.

Safety is a **strategic core domain** of the Construction Compliance Operating System. It is the system of action and evidence for field and site safety work: FLHAs, toolbox talks, inspections, hazards and controls, corrective actions, near misses, incidents, bulletins, permits, lift plans, procedure acknowledgements, risk assessment, daily logs, photos, weather context, and sealed digital signatures—executed through durable workflows with offline support and full auditability.

**Documentation only — no application code.**

---

## 2. Bounded Context

### 2.1 Context Card

| Field | Value |
| --- | --- |
| **Name** | Safety Operations |
| **Module** | `safety` |
| **Strategic type** | Core domain (differentiating) |
| **Product metaphor** | Safety work = assignable, sealable, reviewable compliance activities that produce proof |
| **System of record for** | Safety activities (by type), hazard/control *usage on activities*, tenant hazard/control libraries, corrective actions, incidents/near misses, safety bulletins, permits, lift plans, risk assessments, daily logs, safety photo attachments (as activity evidence refs), weather snapshots on activities, safety reporting projections owned by Safety |
| **Not system of record for** | Controlled document binaries/versions (Documents), signature evidence packages (Signatures), file bytes (Core Files), project lifecycle (Projects), person profiles (People), AuthZ (Core), notification delivery (Notifications), Temporal timers (Workflows platform) |

### 2.2 Context Map Position

```text
Core (authz, membership, files, audit)
Projects (place, required form bindings)
People (PersonId, fit-for-work signal)
        │
        ▼
┌───────────────────────────────────────────┐
│                 SAFETY                    │
│  Activities · Libraries · CA · Incidents  │
│  Bulletins · Permits · Lift · Risk · Logs │
└───────────────┬───────────────────────────┘
                │
    Documents (SWP/SJP controlled copies)
    Signatures (seal packages)
    Equipment (asset refs on permits/lifts)
    Training (eligibility queries)
    Workflows / Notifications / COR / Analytics
```

### 2.3 Ubiquitous Language

| Term | Meaning |
| --- | --- |
| **Safety Activity** | Instance of safety work (FLHA, toolbox, inspection, etc.) |
| **Activity Type** | Configurable definition + schema + workflow binding |
| **FLHA** | Field Level Hazard Assessment (activity subtype) |
| **Toolbox Talk** | Crew talk with attendance/acknowledgement |
| **Site Inspection** | Structured site inspection activity |
| **Hazard** | Identified unwanted energy/event source |
| **Control** | Measure that mitigates a hazard |
| **Hazard Library / Control Library** | Reusable tenant catalogs |
| **Corrective Action (CA)** | Tracked remediation with owner and due date |
| **Near Miss** | Event that could have caused harm but did not |
| **Incident** | Event that caused or may have caused harm; investigated |
| **Safety Bulletin** | Directed safety communication requiring acknowledgement |
| **Permit** | Controlled authorization to perform high-risk work |
| **Lift Plan** | Planned lift with hazards/controls/sign-offs |
| **SWP / SJP** | Safe Work / Safe Job Procedure — *controlled documents*; Safety binds acknowledgements and usage |
| **Risk Matrix** | Likelihood × severity model producing risk rating |
| **Daily Log** | Day/shift safety/operations log for a project |
| **Proof / Sealed** | Activity reached signature-complete evidence state |

---

## 3. Responsibilities (Mapped to Ownership)

| Responsibility | Safety owns? | Clarification |
| --- | --- | --- |
| **FLHAs** | Yes | Activity type + instances |
| **Toolbox Talks** | Yes | Activity + attendance/ack |
| **Site Inspections** | Yes | Activity type + instances |
| **Hazards** | Yes | Library + in-activity hazard entries |
| **Controls** | Yes | Library + in-activity control entries |
| **Corrective Actions** | Yes | CA aggregate + workflow |
| **Near Misses** | Yes | Report activity / case pathway |
| **Incidents** | Yes | IncidentCase + investigation |
| **Safety Bulletins** | Yes | Bulletin aggregate + ack tracking (doc body may be Documents/Files) |
| **Permits** | Yes | Permit-to-work style activities/cases |
| **Lift Plans** | Yes | Lift plan activities/cases; equipment refs foreign |
| **Safe Work Procedures** | Binding + ack | **Documents** owns controlled SWP; Safety requires/records usage & acknowledgement |
| **Safe Job Procedures** | Binding + ack | Same as SWP |
| **Photo Attachments** | Evidence refs | Safety stores attachment refs; **Core Files** store bytes; optional Documents if controlled |
| **Weather** | Snapshot on activity | Observed/fetched weather context stored with activity; not a meteorology platform |
| **Digital Signatures** | Requests + status | **Signatures** owns evidence packages; Safety subjects reference them |
| **Risk Matrix** | Yes | Tenant matrix definition + ratings on assessments |
| **Daily Logs** | Yes | Daily log aggregate per project/shift |

---

## 4. Aggregate Roots

| Aggregate | Responsibility |
| --- | --- |
| **ActivityTypeDefinition** | Catalog of activity kinds (FLHA, Toolbox, Inspection, NearMiss, Permit, LiftPlan, …), schema, workflow key, signature policy ref, offline allowlist |
| **SafetyActivity** | Single runnable instance with status, participants, hazards/controls, responses, attachments, weather, risk rating, signature package refs |
| **HazardLibraryItem** | Reusable hazard definition |
| **ControlLibraryItem** | Reusable control definition |
| **RiskMatrixDefinition** | Tenant (or region) likelihood/severity matrix |
| **CorrectiveAction** | Remediation lifecycle |
| **IncidentCase** | Investigation wrapper for incidents (and optionally serious near misses) |
| **SafetyBulletin** | Bulletin publication + audience + acknowledgement state |
| **PermitCase** | Permit-to-work lifecycle (may specialize SafetyActivity or stand alone when long-lived) |
| **LiftPlanCase** | Lift plan lifecycle (same specialization choice) |
| **DailyLog** | Project/shift daily safety log |
| **SafetyProcedureBinding** | Project/activity requirement linking SWP/SJP document versions |
| **SafetyReportingProjection** | Reporting read models (optional dedicated projection store) |

### 4.1 Modeling Guidance: Activity vs Case

| Pattern | Use when |
| --- | --- |
| **SafetyActivity only** | Short-lived forms: FLHA, toolbox, inspection, simple near miss report, daily checklist |
| **Case + linked activities** | Long-lived: Incident investigation, multi-day permit, complex lift plan with revisions |

Permits and Lift Plans may be implemented as **typed cases** that embed or reference one or more `SafetyActivity` revisions while keeping a stable case id for workflows and COR evidence.

---

## 5. Entities

### 5.1 Under SafetyActivity

| Entity | Description |
| --- | --- |
| **ActivityParticipant** | Person involved (worker, supervisor, visitor) |
| **AttendanceEntry** | Present/ack for toolbox-style activities |
| **HazardEntry** | Hazard applied on this activity (library ref + local notes) |
| **ControlEntry** | Control applied; linked to hazard entries |
| **ActivitySection / ResponseEntry** | Structured answers per type schema |
| **RiskAssessmentEntry** | Likelihood, severity, residual risk using matrix |
| **AttachmentRef** | `FileObjectId` + caption/kind (photo, video still, file) |
| **WeatherSnapshot** | Temp, wind, precip, source, observed_at |
| **ProcedureAckRef** | SWP/SJP document version acknowledged in context |
| **SignaturePackageRef** | Link to Signatures package + role (author, crew, reviewer) |
| **EquipmentRef** | Optional asset ids (Equipment module) |
| **AreaRef** | Optional project area id |
| **OfflineMutationMeta** | Client mutation id, device meta, base version |

### 5.2 Under Libraries

| Entity | Description |
| --- | --- |
| **HazardCategory** | Grouping (optional under library item or separate) |
| **ControlCategory** | Engineering/Admin/PPE etc. |
| **LibraryTag** | Trade/region tags for filtering |
| **HazardControlSuggestion** | Optional recommended controls for a hazard |

### 5.3 Under CorrectiveAction

| Entity | Description |
| --- | --- |
| **ActionAssignment** | Owner person/user |
| **ActionUpdate** | Progress note |
| **ActionAttachmentRef** | Evidence of fix |
| **VerificationRecord** | Close-out verification |

### 5.4 Under IncidentCase

| Entity | Description |
| --- | --- |
| **InvestigationStep** | Workflow steps |
| **InvolvedPerson** | People involved/injured/witness |
| **ContributingFactor** | Analysis factors |
| **LinkedActivityRef** | Source near miss/FLHA/etc. |
| **LinkedCorrectiveActionRef** | CAs spawned |
| **RegulatoryFlag** | Reportable indicators (careful legal config) |

### 5.5 Under Bulletin / Permit / Lift / DailyLog

| Entity | Parent | Description |
| --- | --- | --- |
| **BulletinAudience** | SafetyBulletin | Project/company/role targeting |
| **BulletinAcknowledgement** | SafetyBulletin | Person ack + signature ref |
| **PermitCondition** | PermitCase | Mandatory conditions |
| **PermitHolder** | PermitCase | Authorized persons |
| **LiftComponent** | LiftPlanCase | Load, crane/asset refs, radius, etc. |
| **DailyLogEntry** | DailyLog | Line items / notes / linked activities |

### 5.6 Under RiskMatrixDefinition

| Entity | Description |
| --- | --- |
| **LikelihoodLevel** | Ordered levels with labels |
| **SeverityLevel** | Ordered levels with labels |
| **RiskCell** | Rating outcome per pair (Low/Med/High/Critical…) |

---

## 6. Value Objects

- `SafetyActivityId`, `ActivityTypeId`, `CorrectiveActionId`, `IncidentCaseId`
- `HazardLibraryItemId`, `ControlLibraryItemId`, `RiskMatrixId`
- `BulletinId`, `PermitCaseId`, `LiftPlanCaseId`, `DailyLogId`
- `ProjectId`, `PersonId`, `TeamId`, `AreaId`, `AssetId`, `DocumentVersionId`, `FileObjectId`, `SignaturePackageId`
- `ActivityStatus` — Draft | InProgress | Submitted | UnderReview | Closed | Voided
- `CaseStatus` — Open | Investigating | PendingActions | Closed | Voided
- `CAStatus` — Open | InProgress | PendingVerification | Completed | Overdue | Closed | Cancelled
- `Severity`, `Likelihood`, `RiskRating`
- `HazardCategoryCode`, `ControlType` — Elimination | Substitution | Engineering | Administrative | PPE
- `PermitType`, `LiftClass`
- `WeatherSource` — Manual | Service
- `OfflineSyncState`
- `ClosureReason`, `VoidReason`
- `DueDate`, `EffectivePeriod`

---

## 7. Relationships

```text
Project (Projects)
  └── required form bindings / controls
        │
        ▼
ActivityTypeDefinition (Safety)
        │ instantiates
        ▼
SafetyActivity ──participants──► Person (People)
      │
      ├── HazardEntry ──► HazardLibraryItem
      ├── ControlEntry ──► ControlLibraryItem
      ├── RiskAssessmentEntry ──► RiskMatrixDefinition
      ├── AttachmentRef ──► FileObject (Core)
      ├── ProcedureAckRef ──► DocumentVersion (Documents)  [SWP/SJP]
      ├── SignaturePackageRef ──► SignaturePackage (Signatures)
      ├── EquipmentRef ──► Asset (Equipment)
      ├── WeatherSnapshot
      └── may spawn ──► CorrectiveAction
                      └── IncidentCase

SafetyBulletin ──ack──► Person (+ optional SignaturePackage)
PermitCase / LiftPlanCase ──link──► SafetyActivity revisions, Assets, Documents
DailyLog ──link──► SafetyActivities / notes for Project+Date

CorrectiveAction ← workflow timers (Temporal)
IncidentCase ← investigation workflow
```

### 7.1 SWP / SJP Relationship

```text
Documents: owns SWP/SJP Document + Version (controlled copy)
Safety:    SafetyProcedureBinding + ProcedureAckRef on activities/bulletins
Signatures: seals acknowledgement when policy requires
```

Safety does **not** fork document versioning.

### 7.2 Reusable Libraries Relationship

```text
HazardLibraryItem 1──* HazardControlSuggestion *──1 ControlLibraryItem

SafetyActivity.HazardEntry copies library id + denormalized label snapshot
  so historical activities remain intelligible if library items change
```

---

## 8. Reusable Hazard Library

### 8.1 Purpose

Provide a tenant-curated catalog so field users pick consistent hazards quickly (trade- and region-filterable).

### 8.2 Contents

| Field concept | Notes |
| --- | --- |
| Code / name / description | Required |
| Category | Fall, electrical, mobile equipment, etc. |
| Default severity hint | Optional |
| Suggested controls | Links to control library |
| Trade/region tags | Filtering |
| Status | Active / Retired |
| Versioning | In-place update with snapshot-on-use |

### 8.3 Rules

1. Retiring a library item does not mutate historical `HazardEntry` snapshots.  
2. Custom free-text hazards allowed when policy permits; may require supervisor review.  
3. Library management is permissioned separately from field activity create.  
4. Projects may **prefer** subsets via Projects required controls / settings—but library SoR remains Safety.

---

## 9. Reusable Controls Library

### 9.1 Purpose

Catalog of mitigations aligned to hierarchy of controls.

### 9.2 Contents

| Field concept | Notes |
| --- | --- |
| Code / name / description | Required |
| Control type | Elimination → PPE hierarchy |
| Verification hint | How to confirm control in place |
| Linked hazards | Optional suggestions inverse |
| Status | Active / Retired |

### 9.3 Rules

1. Controls on an activity should reference library ids when picked from catalog.  
2. Residual risk should be reassessed when controls change (business rule on submit).  
3. PPE-only control sets on Critical residual risk may trigger warning or hard gate via tenant settings.

---

## 10. Domain Events

### 10.1 Activity Lifecycle

- `SafetyActivityOpened`
- `SafetyActivityUpdated`
- `SafetyActivitySubmitted`
- `SafetyActivityReviewRequested`
- `SafetyActivityReviewed`
- `SafetyActivityClosed`
- `SafetyActivityVoided`
- `SafetyActivitySignatureRequested`
- `SafetyActivitySealed` *(all required signatures complete)*
- `AttendanceRecorded`
- `WeatherSnapshotRecorded`
- `AttachmentAdded`
- `ProcedureAcknowledged`

### 10.2 Libraries & Risk Matrix

- `HazardLibraryItemDefined` / `Updated` / `Retired`
- `ControlLibraryItemDefined` / `Updated` / `Retired`
- `RiskMatrixPublished` / `Retired`

### 10.3 Corrective Actions

- `CorrectiveActionOpened`
- `CorrectiveActionAssigned`
- `CorrectiveActionUpdated`
- `CorrectiveActionOverdue`
- `CorrectiveActionCompleted`
- `CorrectiveActionVerified`
- `CorrectiveActionClosed`
- `CorrectiveActionCancelled`

### 10.4 Incidents & Near Misses

- `NearMissReported`
- `IncidentCaseOpened`
- `IncidentInvestigationUpdated`
- `IncidentCaseClosed`
- `CriticalRiskRaised`

### 10.5 Bulletins, Permits, Lifts, Daily Logs

- `SafetyBulletinPublished`
- `SafetyBulletinAcknowledged`
- `SafetyBulletinClosed`
- `PermitRequested` / `PermitIssued` / `PermitSuspended` / `PermitClosed`
- `LiftPlanCreated` / `LiftPlanApproved` / `LiftPlanCompleted` / `LiftPlanVoided`
- `DailyLogOpened` / `DailyLogUpdated` / `DailyLogClosed`

### 10.6 Envelope

Includes `tenant_id`, `project_id` (when applicable), actor, correlation IDs, activity/case ids. No signature bitmap payloads; no raw medical PHI from People.

---

## 11. Commands & Queries (Public Interfaces)

### 11.1 Command Examples

| Command | Aggregate |
| --- | --- |
| `DefineActivityType` / `PublishActivityType` | ActivityTypeDefinition |
| `StartSafetyActivity` / `UpdateSafetyActivity` / `SubmitSafetyActivity` | SafetyActivity |
| `RecordAttendance` / `AddHazardEntry` / `AddControlEntry` | SafetyActivity |
| `AttachPhoto` / `RecordWeatherSnapshot` | SafetyActivity |
| `RequestSignatures` / `MarkActivitySealed` *(on Signatures callback)* | SafetyActivity |
| `ReviewSafetyActivity` / `CloseSafetyActivity` / `VoidSafetyActivity` | SafetyActivity |
| `DefineHazardLibraryItem` / `DefineControlLibraryItem` | Libraries |
| `PublishRiskMatrix` | RiskMatrixDefinition |
| `OpenCorrectiveAction` / `AssignCorrectiveAction` / `CompleteCorrectiveAction` / `VerifyCorrectiveAction` | CorrectiveAction |
| `OpenIncidentCase` / `UpdateInvestigation` / `CloseIncidentCase` | IncidentCase |
| `PublishSafetyBulletin` / `AcknowledgeBulletin` | SafetyBulletin |
| `RequestPermit` / `IssuePermit` / `ClosePermit` | PermitCase |
| `CreateLiftPlan` / `ApproveLiftPlan` / `CompleteLiftPlan` | LiftPlanCase |
| `OpenDailyLog` / `AppendDailyLogEntry` / `CloseDailyLog` | DailyLog |
| `BindSafetyProcedure` | SafetyProcedureBinding |

### 11.2 Query API (`SafetyQueryApi`)

| Query | Consumers |
| --- | --- |
| `GetActivity` / `ListActivities(filter)` | UI, COR, Analytics |
| `ListOpenCorrectiveActions` | Command Center, Projects dashboard |
| `GetIncidentCase` | Safety coordinators |
| `ListLibraryHazards` / `ListLibraryControls` | Field UI |
| `GetRiskMatrix` | Activity risk UX |
| `GetProjectSafetyStats` | Projects dashboard projector / Analytics |
| `ListPendingAcknowledgements(person)` | My Actions |
| `GetPermit` / `GetLiftPlan` / `GetDailyLog` | UI, Equipment |
| `AssertActivityCloseable` | Workflows |

### 11.3 HTTP Surface (Illustrative)

Base: `/api/safety`

- `/activity-types`, `/activities`, `/activities/{id}/submit|review|close|void`
- `/hazards/library`, `/controls/library`, `/risk-matrices`
- `/corrective-actions`, `/incidents`, `/near-misses`
- `/bulletins`, `/permits`, `/lift-plans`, `/daily-logs`
- `/reports/...` (read models)

All routes: Core AuthN/AuthZ + project membership gates as required.

---

## 12. Business Rules (Selected)

### 12.1 Activity Rules

1. Activities require valid active `ProjectId` (Projects query) unless tenant allows non-project drafts (rare).  
2. Status transitions are forward-only except admin void; no silent reopen without permission.  
3. Submit requires mandatory schema sections complete.  
4. FLHA/Toolbox typically require ≥1 hazard and mapped controls before submit (configurable by type).  
5. Residual risk Critical may require elevated reviewer role.  
6. Closed activities are immutable except void with reason.  
7. Snapshots of hazard/control labels are stored on entries at selection time.

### 12.2 Signature Rules

1. When type requires signatures, activity cannot reach `Closed`/`Sealed` until Signatures reports package complete.  
2. Safety calls Signatures to create packages; does not store stroke geometry.  
3. Crew toolbox: multi-signer package; progress visible in UI.  
4. Guest signing allowed only if Core/Signatures policy permits for that type.

### 12.3 Corrective Actions

1. CA must reference source activity/case or explicit manual source.  
2. Due dates required; overdue emits event via Temporal watcher.  
3. Completion may require verification step by policy.  
4. Closing incident may require all linked CAs closed (configurable).

### 12.4 Incidents & Near Misses

1. Near miss can be upgraded to IncidentCase.  
2. Incident investigation steps follow workflow definition.  
3. `CriticalRiskRaised` notifies aggressively (Notifications + forced channels).  
4. Regulatory reportability flags are configuration-driven by region—not hard-coded legal advice.

### 12.5 Permits & Lift Plans

1. Issue/approve requires authorized roles + sealed signatures when configured.  
2. Suspend on overdue CA / failed equipment readiness if linked asset not ready (Equipment query).  
3. Lift plans must reference equipment assets when type requires; readiness is Equipment’s decision.

### 12.6 SWP / SJP

1. Binding points at Documents `DocumentVersionId`.  
2. Acknowledgement creates Documents ack and/or Signatures package per policy.  
3. Using an obsolete version is rejected (Documents effective-version query).

### 12.7 Weather & Photos

1. Weather is contextual evidence, not a standalone compliance object.  
2. Photos go through Core `FileApi` (quarantine/scan); Safety stores refs + captions.  
3. Offline photos queue with mutation ids; become Available after sync/complete upload.

### 12.8 Daily Logs

1. One open daily log per project+date+shift scope (tenant uniqueness rule).  
2. Closing log may snapshot counts of activities/CAs for the day (projection aids).

### 12.9 Eligibility Collaboration

Before starting restricted activities, application layer may compose:

```text
Core membership + People active/fit signal + Training requirements
  + Documents acks + Equipment readiness (if asset-linked)
```

Safety owns the **activity decision to start/submit**, not the foreign modules’ rules.

---

## 13. Workflow Integration

### 13.1 Role of Temporal

Durable processes for Safety:

| Workflow | Purpose |
| --- | --- |
| `SafetyActivityReviewWorkflow` | Submit → review → seal → close; reminders/escalations |
| `ToolboxAcknowledgementWorkflow` | Multi-signer completion tracking |
| `CorrectiveActionSlaWorkflow` | Due timers; overdue; escalate |
| `IncidentInvestigationWorkflow` | Step gates; assignments; close criteria |
| `PermitLifecycleWorkflow` | Request → issue → expiry/suspend → close |
| `LiftPlanApprovalWorkflow` | Multi-party approve before lift |
| `BulletinAckWorkflow` | Audience completion; nag; close |
| `DailyLogReminderWorkflow` | Optional end-of-shift reminders |

### 13.2 Rules

1. Workflows **orchestrate**; Safety aggregates **decide** invariants.  
2. Activities (Temporal) call Safety and Core/Signatures public APIs only.  
3. Never bypass Temporal for CA SLA, multi-signer completion, or permit expiry.  
4. Workflow visibility projected into My Actions / activity status.

### 13.3 Sequence (FLHA)

```text
StartSafetyActivity
  → update hazards/controls/risk/photos (online or offline queue)
  → SubmitSafetyActivity
  → start SafetyActivityReviewWorkflow
  → RequestSignatures (Signatures)
  → on sealed: Review/Close
  → events → Notifications, COR, Projects dashboard, Analytics
  → Core.AuditApi on each significant transition
```

---

## 14. Permissions

Registered in Core; enforced on every command.

| Code | Intent |
| --- | --- |
| `safety.activity.read` | View activities |
| `safety.activity.create` | Start FLHA/toolbox/inspection/etc. |
| `safety.activity.submit` | Submit |
| `safety.activity.review` | Review/approve |
| `safety.activity.close` | Close |
| `safety.activity.void` | Void |
| `safety.library.manage` | Hazard/control libraries |
| `safety.risk_matrix.manage` | Matrix admin |
| `safety.ca.manage` | Create/assign/update CAs |
| `safety.ca.verify` | Verify completion |
| `safety.incident.manage` | Incident investigation |
| `safety.bulletin.publish` | Publish bulletins |
| `safety.bulletin.ack` | Acknowledge |
| `safety.permit.manage` / `safety.permit.issue` | Permits |
| `safety.lift.manage` / `safety.lift.approve` | Lift plans |
| `safety.dailylog.manage` | Daily logs |
| `safety.reports.read` | Safety reporting |
| `safety.procedure.bind` | Bind SWP/SJP requirements |

Scopes: typically **Project**; library/matrix often **Tenant**.

---

## 15. Offline Support

### 15.1 Allowlisted Offline Mutations

| Allowed offline (typical) | Not offline (initial) |
| --- | --- |
| Create/update draft FLHA, toolbox, inspection | Incident regulatory close-out |
| Add hazards/controls/responses | Library administration |
| Attach photo intents (local → sync upload) | Permit issue for critical types (config) |
| Record weather manual snapshot | Void after sealed |
| Submit (queue) where type allows | Membership/permission changes |

### 15.2 Sync Rules

1. Idempotent commands keyed by `mutation_id`.  
2. Server validates project active, membership, schema, and eligibility on sync.  
3. Conflicts: reject illegal transitions; preserve server sealed state as winner.  
4. Signature capture offline only if Signatures module policy supports offline seal + sync (otherwise sign online).  
5. UX shows Saved on device → Syncing → Proven (per UX architecture).

### 15.3 Reference Data Offline

Cached subsets: hazard/control libraries, activity type schemas, project areas, recent crew lists—**snapshots with staleness**, not AuthZ authority.

---

## 16. Audit Trail

### 16.1 Dual Layer

| Layer | Owner | Purpose |
| --- | --- | --- |
| **Core Audit** | Core | Security/compliance-significant action log (who/when/what/resource) |
| **Safety domain history** | Safety | Activity/case status timeline for operational UX |

### 16.2 Must Audit via Core

- Submit, review, close, void  
- CA assign/complete/verify  
- Incident open/close  
- Permit issue/suspend  
- Lift approve  
- Bulletin publish  
- Library retire (tenant admin)  
- Procedure bind  

### 16.3 Provenance for COR

Events + signature package ids + file object ids + document version ids form evidence provenance consumed by COR without Safety exporting raw tables.

---

## 17. Notifications

Safety emits domain events; **Notifications** module decides delivery.

| Event class | Typical notify |
| --- | --- |
| Assigned review / CA | In-app + push/email |
| Overdue CA | Escalation channels |
| CriticalRisk / Incident opened | Forced critical policy |
| Bulletin published | Audience members |
| Permit expiring | Holders + supervisors |
| Signature waiting | Pending signers |
| Daily log reminder | Supervisors |

Safety does not send email directly. Workers’ preference ceilings still apply except forced critical tenant policies.

---

## 18. Reporting

### 18.1 Layers

| Layer | Store | Use |
| --- | --- | --- |
| Operational lists | Postgres Safety | Open CAs, today’s activities |
| Project dashboard stats | Projects projections ← Safety events | Place overview |
| Safety reports | Safety reporting projections | Safety coordinator reports |
| Portfolio analytics | ClickHouse ← events | Trends, hotspots |

### 18.2 Example Report Sets

- FLHA completion rates by project/trade  
- Toolbox attendance/seal rates  
- CA aging and overdue  
- Incident/near miss frequency  
- Permit cycle time  
- Risk rating distributions  
- Bulletin acknowledgement completion  

### 18.3 Rules

1. Reports are read models—**not** alternate write authorities.  
2. Rebuildable from events for defined windows.  
3. RBAC scopes all report reads.  
4. No PHI from People medical details in Safety warehouses.

---

## 19. Data Ownership

### 19.1 Schema `safety` Owns

- Activity types, activities, participants, hazard/control entries  
- Hazard & control libraries, risk matrices  
- Corrective actions, incidents/near miss cases  
- Bulletins, permits, lift plans, daily logs  
- Procedure bindings (refs), attachment refs, weather snapshots  
- Safety reporting projections  

### 19.2 Foreign Ownership Reminder

| Data | Owner |
| --- | --- |
| File bytes | Core Files |
| SWP/SJP versions | Documents |
| Signature evidence | Signatures |
| Asset readiness | Equipment |
| Membership/authz | Core |
| Person profile | People |
| Project status | Projects |

---

## 20. Integration Summary (Other Modules)

| Module | Interaction |
| --- | --- |
| **Core** | Authorize every command; membership; files; audit append; flags/license |
| **Projects** | Active project gate; form bindings drive required activity types; consume Safety stats events |
| **People** | `PersonId` participants; fit-for-work coarse signal before high-risk work |
| **Documents** | SWP/SJP versions; bulletin bodies if controlled; ack collaboration |
| **Signatures** | Create/track packages; seal callback |
| **Equipment** | Asset refs on permits/lifts/inspections; readiness queries |
| **Training** | Eligibility for restricted activity types |
| **Workflows** | SLA, review, multi-sign, permit expiry |
| **Notifications** | Fan-out from events |
| **COR / Analytics** | Evidence and trends from Safety events |
| **Web/PWA** | My Actions + Place Safety tabs; offline queues |

---

## 21. Anti-Patterns

1. Storing signature strokes in Safety  
2. Forking SWP/SJP document versions inside Safety  
3. Implementing CA timers only in the client  
4. Treating People attendance as toolbox proof  
5. Mutating hazard library text in historical activities without snapshots  
6. SQL joins into Core/Documents schemas  
7. Dashboard stats as editable “truth”  
8. Bypassing Core audit on void/close/issue  

---

## 22. Success Criteria

Safety is correctly designed when:

1. Field crews can complete FLHAs/toolboxes offline and end in **sealed proof**.  
2. Hazards/controls are consistent via libraries yet historically stable.  
3. CAs and incidents escalate reliably through Temporal.  
4. SWP/SJP and signatures remain correctly owned by Documents/Signatures.  
5. Projects Command Center and COR can trust Safety events as evidence sources.  
6. Permissions and audit make high-risk actions defensible.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Principal Architecture | Initial Safety domain architecture |

---

*End of Safety Domain Architecture*
