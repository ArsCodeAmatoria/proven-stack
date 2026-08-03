# Proven — Equipment Domain Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Equipment Domain Architecture (DDD) |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Principal Architecture |
| **Audience** | Engineering, Product, Design, Equipment / Crane Operations |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Domain Model](./DOMAIN_MODEL.md), [Core Domain](./CORE_DOMAIN.md), [Projects Domain](./PROJECTS_DOMAIN.md), [People Domain](./PEOPLE_DOMAIN.md), [Safety Domain](./SAFETY_DOMAIN.md), [System Architecture](./SYSTEM_ARCHITECTURE.md), [PRD](../PRD.md) |

---

## 1. Purpose

This document defines the **Equipment** bounded context for Proven.

Equipment is a **strategic core domain** of the Construction Compliance Operating System. It is the system of record for assets—from tower cranes and mobile cranes through rigging, forklifts, telehandlers, vehicles, generators, and tools—and for the inspections, deficiencies, maintenance, certifications, binders, photos, and QR identity that prove an asset is ready for use.

**Documentation only — no application code.**

---

## 2. Bounded Context

### 2.1 Context Card

| Field | Value |
| --- | --- |
| **Name** | Equipment Compliance |
| **Module** | `equipment` |
| **Strategic type** | Core domain (differentiating) |
| **Product metaphor** | Asset = governed equipment identity with readiness proof |
| **System of record for** | Assets (all classes), asset types/profiles, assignments/custody, pre-use & periodic inspections, deficiencies, maintenance records/history, equipment certification records, crane binders (tower & self-erect) as equipment dossiers, QR identity bindings, equipment photo refs, readiness state, equipment reporting projections |
| **Not system of record for** | File bytes (Core Files), controlled document versioning semantics (Documents), signature evidence packages (Signatures), lift plan *safety case* logic (Safety), project lifecycle (Projects), operator HR profiles (People), AuthZ (Core), notification delivery (Notifications) |

### 2.2 Context Map

```text
Core (authz, files, audit) · Projects (place) · People (operators)
        │
        ▼
┌────────────────────────────────────────────┐
│                 EQUIPMENT                  │
│  Assets · Inspections · Deficiencies       │
│  Maintenance · Certs · Binders · QR/Photos │
└──────────────────┬─────────────────────────┘
                   │
     Safety (lift/permit asset refs) · Documents (cert PDFs as controlled docs)
     Signatures (inspection sign-off) · Training (operator competency queries)
     Notifications · Workflows · COR · Analytics · Projects dashboard
```

### 2.3 Ubiquitous Language

| Term | Meaning |
| --- | --- |
| **Asset** | Tracked equipment item with identity and readiness |
| **Asset Class** | Tower Crane, Mobile Crane, Rigging, Forklift, Telehandler, Vehicle, Generator, Tool, Other |
| **Readiness** | Ready / Restricted / Blocked — decision signal for use |
| **Pre-Use Inspection** | Shift/use checklist inspection before operation |
| **Periodic Inspection** | Scheduled formal inspection |
| **Deficiency** | Defect/finding that may restrict or block use |
| **Maintenance Record** | Work performed or planned on an asset |
| **Certification Record** | Time-bounded credential/evidence for asset compliance |
| **Binder** | Structured dossier of required documents/inspections for crane classes |
| **Tower Crane Binder** | Binder profile for tower cranes |
| **Self-Erect Binder** | Binder profile for self-erecting cranes |
| **QR Code** | Scannable identity binding to `AssetId` |
| **Custody** | Who/where the asset is assigned |

---

## 3. Responsibilities (Mapped to Ownership)

| Responsibility | Equipment owns? | Clarification |
| --- | --- | --- |
| **Tower Cranes** | Yes | Asset class + tower-specific profile + binder |
| **Mobile Cranes** | Yes | Asset class + mobile profile |
| **Rigging** | Yes | Assets / kits / components as configured |
| **Forklifts** | Yes | Asset class |
| **Telehandlers** | Yes | Asset class |
| **Vehicles** | Yes | Asset class |
| **Generators** | Yes | Asset class |
| **Tools** | Yes | Asset class (serialized or pooled per policy) |
| **Assets** | Yes | Canonical registry |
| **Maintenance** | Yes | Maintenance records & schedules; not full CMMS ERP replacement |
| **Deficiencies** | Yes | Findings from inspections/maintenance |
| **Pre-Use Inspections** | Yes | Inspection instances + checklists |
| **Periodic Inspections** | Yes | Inspection instances + due schedules |
| **Tower Crane Binder** | Yes | Binder aggregate / dossier checklist |
| **Self-Erect Binder** | Yes | Binder aggregate / dossier checklist |
| **Maintenance History** | Yes | Historical maintenance timeline |
| **Certifications** | Yes (records) | May reference Documents/Files for binaries |
| **Photos** | Refs | Core Files store bytes; Equipment stores photo refs |
| **QR Codes** | Yes | Identity binding + resolution API |

---

## 4. Aggregate Roots

| Aggregate | Responsibility |
| --- | --- |
| **Asset** | Identity, class, status, ownership, assignment, QR binding, class-specific profile data, readiness |
| **AssetTypeDefinition** | Tenant catalog of types within classes (checklists, binder templates, inspection intervals) |
| **InspectionChecklistDefinition** | Reusable checklist schema for pre-use/periodic by type |
| **Inspection** | A performed pre-use or periodic inspection instance |
| **Deficiency** | Defect lifecycle until cleared or accepted risk per policy |
| **MaintenanceOrder** | Planned/completed maintenance work against an asset |
| **CertificationRecord** | Certification/credential with issue/expiry and evidence refs |
| **EquipmentBinder** | Crane binder instance (tower or self-erect) tracking required sections/completeness |
| **BinderTemplate** | Required sections for tower vs self-erect binders |
| **EquipmentReportingProjection** | Read models for fleet reports (optional) |

---

## 5. Entities

### 5.1 Under Asset

| Entity | Description |
| --- | --- |
| **AssetIdentity** | Tag, serial, manufacturer, model, year |
| **AssetOwnership** | Owning `CompanyId`, optional customer/client refs |
| **AssetAssignment** | Current project/person/location assignment |
| **CustodyTransfer** | History of custody changes |
| **QrBinding** | QR payload / code id ↔ AssetId |
| **PhotoRef** | `FileObjectId` + caption/kind |
| **ClassProfile** | Type-specific attributes (see §5.6) |
| **ReadinessSnapshot** | Cached derived readiness + reasons (rebuildable) |

### 5.2 Under Inspection

| Entity | Description |
| --- | --- |
| **ChecklistItemResult** | Pass/Fail/NA + notes |
| **InspectionAttachmentRef** | Photos/files for the inspection |
| **InspectorRef** | `PersonId` / principal |
| **SignaturePackageRef** | Sign-off via Signatures |
| **DeficiencySpawnRef** | Deficiencies created from failed items |
| **OfflineMutationMeta** | Idempotent sync metadata |

### 5.3 Under Deficiency

| Entity | Description |
| --- | --- |
| **DeficiencyUpdate** | Progress notes |
| **DeficiencyAttachmentRef** | Evidence |
| **SeverityAssessment** | Impacts readiness |
| **ClearanceRecord** | Who cleared, when, verification |

### 5.4 Under MaintenanceOrder

| Entity | Description |
| --- | --- |
| **MaintenanceTask** | Work items |
| **PartsNote** | Optional parts references |
| **MaintenanceAttachmentRef** | Evidence |
| **LinkedDeficiencyRef** | Deficiencies addressed |
| **VendorRef** | Optional external vendor snapshot |

### 5.5 Under CertificationRecord & Binder

| Entity | Parent | Description |
| --- | --- | --- |
| **EvidenceRef** | CertificationRecord | DocumentVersionId and/or FileObjectId |
| **BinderSection** | EquipmentBinder | Required section (manual, cert, inspection, engineering letter, etc.) |
| **BinderSectionItem** | BinderSection | Linked cert/inspection/file/doc + status Complete/Missing/Expired |
| **BinderSignOff** | EquipmentBinder | Optional supervisory seal |

### 5.6 Class Profiles (Entity or VO sets on Asset)

| Class | Profile fields (illustrative) |
| --- | --- |
| **Tower Crane** | Tower height, jib length, capacity, base type, climbing config, manufacturer specs refs |
| **Mobile Crane** | Capacity, boom config, carrier type, axle load notes |
| **Self-Erect** | Model family, max height/reach, erect sequence doc refs |
| **Rigging** | WLL, length, material, inspection color period, kit membership |
| **Forklift / Telehandler** | Capacity, fuel type, mast/boom, tire type |
| **Vehicle** | VIN/plate, GVWR, class |
| **Generator** | kW, fuel, phase |
| **Tool** | Calibration required flag, pooled vs serialized |

---

## 6. Value Objects

- `AssetId`, `AssetTag`, `SerialNumber`, `QrCodeId`
- `AssetClass`, `AssetTypeId`
- `AssetStatus` — Available | Assigned | OutOfService | Retired
- `ReadinessState` — Ready | Restricted | Blocked
- `ReadinessReasonCode` — ExpiredCert | FailedPreUse | OpenDeficiency | BinderIncomplete | MissingPeriodic | ManualHold | …
- `InspectionKind` — PreUse | Periodic | PostIncident | Other
- `InspectionStatus` — Draft | Submitted | Passed | Failed | Voided
- `DeficiencyStatus` — Open | InRepair | PendingVerification | Cleared | AcceptedDeferred
- `DeficiencySeverity` — Low | Medium | High | Critical
- `MaintenanceStatus` — Planned | InProgress | Completed | Cancelled
- `CertificationStatus` — Valid | Expiring | Expired | Revoked
- `BinderKind` — TowerCrane | SelfErect
- `BinderCompleteness` — Complete | Incomplete | ExpiredItems
- `ProjectId`, `PersonId`, `CompanyId`, `FileObjectId`, `DocumentVersionId`, `SignaturePackageId`
- `ExpiryDate`, `DueDate`, `GeoLocation` (optional site position)

---

## 7. Relationships

```text
Company (Core) ◄── owns ── Asset
Project (Projects) ◄── assigned ── AssetAssignment
Person (People) ◄── operator/custodian ── AssetAssignment / Inspection.Inspector

AssetTypeDefinition ──defines──► checklists, intervals, binder template
Asset ──performs──► Inspection ──may spawn──► Deficiency
Asset ──has──► MaintenanceOrder ──may clear──► Deficiency
Asset ──has──► CertificationRecord ──evidence──► Documents/Files
Asset ──has──► EquipmentBinder (tower/self-erect) ──sections──► certs/inspections/docs
Asset ──has──► QrBinding
Asset ──has──► PhotoRef ──► Core FileObject

Safety LiftPlan/Permit ──references──► AssetId
Projects EquipmentRequirement ──constrains──► AssetClass/Type on project
Training ──operator competency──► queried before assign/operate (not stored as SoR here)
```

### 7.1 Binder Relationship Detail

```text
BinderTemplate (Tower | SelfErect)
        │ instantiates for asset
        ▼
EquipmentBinder
        ├── Section: Manufacturer docs → Document/File refs
        ├── Section: Certifications → CertificationRecord refs
        ├── Section: Periodic inspections → Inspection refs
        ├── Section: Engineering / foundation letters → Document refs
        ├── Section: Maintenance summary → MaintenanceOrder refs
        └── Completeness → feeds ReadinessState
```

---

## 8. Domain Events

### 8.1 Asset Lifecycle

- `AssetRegistered`
- `AssetUpdated`
- `AssetClassProfileUpdated`
- `AssetAssignedToProject`
- `AssetAssignedToPerson`
- `AssetUnassigned`
- `AssetTakenOutOfService`
- `AssetReturnedToService`
- `AssetRetired`
- `AssetQrBound`
- `AssetPhotoAdded`
- `AssetReadinessChanged`

### 8.2 Inspections

- `InspectionStarted`
- `InspectionSubmitted`
- `InspectionPassed`
- `InspectionFailed`
- `InspectionVoided`
- `PreUseInspectionDue` *(workflow/projection)*
- `PeriodicInspectionDue`
- `PeriodicInspectionOverdue`

### 8.3 Deficiencies & Maintenance

- `DeficiencyOpened`
- `DeficiencyUpdated`
- `DeficiencyCleared`
- `DeficiencyDeferred`
- `MaintenanceOrderCreated`
- `MaintenanceOrderCompleted`
- `MaintenanceHistoryAppended`

### 8.4 Certifications & Binders

- `CertificationRecorded`
- `CertificationExpiring`
- `CertificationExpired`
- `CertificationRevoked`
- `BinderCreated`
- `BinderSectionCompleted`
- `BinderSectionExpired`
- `BinderCompletenessChanged`
- `BinderSignedOff`

### 8.5 Envelope

`tenant_id`, `asset_id`, `project_id` (optional), actor, correlation IDs, readiness after-state when relevant.

---

## 9. Business Rules

### 9.1 Identity & Registry

1. `AssetTag` unique per tenant (policy); serial unique within manufacturer when provided.  
2. QR codes bind 1:1 to an active asset; reassignment requires unbind + audit.  
3. Retired assets cannot be assigned or inspected for use (view/history only).  
4. Class profile required fields enforced by `AssetTypeDefinition`.

### 9.2 Assignment

1. Assign to project requires Projects `IsProjectActive` (or allow staging per settings).  
2. Assign to person requires People `AssertPersonActive`.  
3. Operating high-risk classes may require Training competency query before assignment unlock (application policy).  
4. Unassigning does not delete history.

### 9.3 Readiness Derivation

```text
Readiness = Ready
  unless any blocking reason:
    AssetStatus OutOfService/Retired
    OR open Critical deficiency
    OR failed/missing required pre-use within validity window
    OR periodic inspection overdue
    OR required certification expired
    OR binder incomplete/expired for crane classes that require binder
    OR manual hold
  → Blocked (or Restricted if policy maps partial issues to Restricted)
```

1. `GetAssetReadiness` is the public decision query for Safety/Projects.  
2. Readiness is derived and evented (`AssetReadinessChanged`); not freely user-edited.  
3. Manual hold/release requires elevated permission + audit reason.

### 9.4 Pre-Use Inspections

1. Required for configured classes before use each shift/day window.  
2. Failed pre-use → open deficiencies + readiness Blocked/Restricted.  
3. Sign-off via Signatures when type requires; cannot Pass sealed without package complete.  
4. Offline drafts allowed; sync validates intervals and membership.  
5. Passed pre-use has a validity window (e.g., remainder of shift); expiry returns readiness impact.

### 9.5 Periodic Inspections

1. Interval from AssetTypeDefinition (calendar and/or hour meter if tracked).  
2. Overdue periodic → readiness Blocked for classes configured as hard gate.  
3. Periodic failure opens deficiencies and may force OutOfService.

### 9.6 Deficiencies

1. Critical severity blocks readiness until cleared or formally deferred (if policy allows—rare for critical).  
2. Clearing may require verification + signature.  
3. Deficiencies remain in history after clear.

### 9.7 Maintenance

1. Maintenance completion may clear linked deficiencies when verified.  
2. Maintenance history is append-oriented; corrections are new records or void-with-reason.  
3. Equipment is not a full financial CMMS; cost accounting remains out of scope.

### 9.8 Certifications

1. Evidence should reference Documents and/or Core Files.  
2. Expiry workflows emit `CertificationExpiring` / `Expired`.  
3. Expired required cert → readiness Blocked.  
4. Revocation immediate block + audit.

### 9.9 Crane Binders (Tower & Self-Erect)

1. Tower cranes require Tower Crane Binder instance when type mandates.  
2. Self-erecting cranes require Self-Erect Binder when type mandates.  
3. Binder completeness is necessary for Ready on those assets.  
4. Sections map to required document/cert/inspection evidence; missing/expired section → Incomplete.  
5. Binder templates are tenant-manageable within platform policy ceilings.  
6. Engineering letters and manufacturer manuals are document refs—not duplicated blobs in Equipment.

### 9.10 Photos & QR

1. Photos via Core FileApi; Equipment stores refs.  
2. QR resolve API returns asset summary + readiness for field scan UX.  
3. Counterfeit/unknown QR → not found; do not create assets implicitly.

### 9.11 Cross-Module

1. Safety lift plans reference `AssetId`; they must query readiness before approve/perform.  
2. Projects equipment requirements do not assign assets; Equipment owns assignment writes.  
3. People attendance ≠ equipment inspection proof.

---

## 10. Workflows

Temporal-orchestrated processes:

| Workflow | Purpose |
| --- | --- |
| `PreUseValidityWorkflow` | Track validity window; notify on expiry |
| `PeriodicInspectionDueWorkflow` | Schedule due/overdue; escalate |
| `CertificationExpiryWorkflow` | Expiring/expired notices; readiness update commands |
| `DeficiencySlaWorkflow` | Overdue open deficiencies |
| `MaintenanceDueWorkflow` | Planned maintenance reminders |
| `BinderCompletenessWatch` | React to cert/inspection events; recompute binder + readiness |
| `InspectionSignOffWorkflow` | Multi-party seal for periodic inspections when required |
| `OutOfServiceReleaseWorkflow` | Controlled return-to-service verification steps |

### 10.1 Workflow Rules

- Workflows call Equipment public commands/queries and Core/Signatures as needed.  
- Readiness recomputation is a domain command/service after each material event.  
- Never rely on client timers for cert/periodic enforcement.

### 10.2 Example: Failed Pre-Use

```text
StartInspection(PreUse) → fail item
  → InspectionFailed
  → OpenDeficiency(Critical)
  → RecomputeReadiness(Blocked)
  → AssetTakenOutOfService (optional auto)
  → start DeficiencySlaWorkflow
  → Notifications to custodian/supervisor
  → Core.AuditApi
```

### 10.3 Example: Tower Crane Binder Gate

```text
Assign tower crane to Active project
  → ensure EquipmentBinder exists from Tower template
  → evaluate sections
  → if Incomplete: Readiness Blocked + notify equipment manager
  → Safety lift approve queries GetAssetReadiness → deny if Blocked
```

---

## 11. Notifications

Equipment emits events; **Notifications** delivers.

| Trigger | Audience (typical) |
| --- | --- |
| Periodic due/overdue | Equipment managers, supervisors |
| Cert expiring/expired | Equipment managers, project PM |
| Pre-use failed / OutOfService | Custodian, supervisor, project safety |
| Deficiency SLA breach | Owner + escalations |
| Binder incomplete on assigned crane | Crane yard / project |
| Maintenance due | Maintenance assignees |
| Return-to-service complete | Supervisors |

Critical readiness losses on in-use project assets use elevated notification policy.

---

## 12. Reporting

| Report / view | Source |
| --- | --- |
| Fleet inventory by class/status | Equipment Postgres |
| Readiness heatmap by project | Projections + events |
| Overdue periodic inspections | Equipment + workflows |
| Open deficiencies aging | Equipment |
| Certification expiry calendar | CertificationRecord |
| Binder completeness (tower/self-erect) | EquipmentBinder |
| Maintenance history per asset | MaintenanceOrder timeline |
| Pre-use compliance rates | Inspection events → Analytics/ClickHouse |
| Portfolio equipment KPIs | ClickHouse |

Rules: rebuildable projections; RBAC-scoped; readiness reports never override live `GetAssetReadiness` for enforcement.

---

## 13. Permissions

| Code | Intent |
| --- | --- |
| `equipment.asset.read` | View assets |
| `equipment.asset.create` | Register assets |
| `equipment.asset.update` | Update profile |
| `equipment.asset.assign` | Assign/unassign project/person |
| `equipment.asset.retire` | Retire |
| `equipment.asset.hold` | Manual readiness hold/release |
| `equipment.inspection.perform` | Pre-use/periodic perform |
| `equipment.inspection.void` | Void inspection |
| `equipment.deficiency.manage` | Open/update/clear deficiencies |
| `equipment.maintenance.manage` | Maintenance orders |
| `equipment.certification.manage` | Cert records |
| `equipment.binder.manage` | Binder sections/templates |
| `equipment.binder.signoff` | Binder sign-off |
| `equipment.type.manage` | Types/checklists/intervals |
| `equipment.qr.manage` | Bind/unbind QR |
| `equipment.reports.read` | Reports |

Scopes: Tenant for fleet admin; Project for site-assigned operations; Self for operator pre-use on assigned assets (policy).

---

## 14. Public Interfaces & API

### 14.1 In-Process Interfaces

| Interface | Purpose |
| --- | --- |
| `EquipmentQueryApi` | GetAsset, readiness, list by project, resolve QR, binder status |
| `EquipmentCommandApi` | Register/assign/inspect/maintenance commands for workflows |
| `EquipmentReadinessApi` | `GetAssetReadiness(AssetId, ProjectId?)` → state + reasons |

### 14.2 HTTP API (Illustrative)

Base: `/api/equipment`

| Area | Paths |
| --- | --- |
| Assets | `POST/GET/PATCH /assets`, `/assets/{id}/assign`, `/retire`, `/hold` |
| Classes | `GET /assets?class=tower_crane` filters |
| QR | `POST /assets/{id}/qr`, `GET /qr/{code}` resolve |
| Photos | `POST /assets/{id}/photos` (via file intent) |
| Checklists | `/checklist-definitions` |
| Inspections | `POST /inspections`, `/inspections/{id}/submit` |
| Deficiencies | `/deficiencies` |
| Maintenance | `/maintenance-orders` |
| Certifications | `/certifications` |
| Binders | `/binders`, `/binder-templates`, `/binders/{id}/sections` |
| Reports | `/reports/readiness`, `/reports/overdue-inspections`, … |

All routes: Core AuthN/AuthZ; significant writes → Core Audit.

### 14.3 Key Query Contracts

**`GetAssetReadiness`**

- Input: `AssetId`, optional `ProjectId`, optional `asOf`  
- Output: `ReadinessState`, `reasons[]`, `validPreUseUntil?`, `binderCompleteness?`, `nextPeriodicDue?`

**`ResolveQr`**

- Input: QR payload  
- Output: asset summary + readiness + deep link ids  

---

## 15. Offline Support

| Allow offline | Online-only (initial) |
| --- | --- |
| Pre-use inspection drafts/submit for assigned assets | Binder template admin |
| Photo capture queue | Asset registry create for cranes (optional allow) |
| Deficiency note drafts | Cert expiry admin corrections |
| QR resolve against cached assigned fleet | Cross-tenant operations |

Idempotent `mutation_id`; server recomputes readiness on sync; sealed inspections require Signatures policy compliance.

---

## 16. Audit Trail

Must Core-audit:

- Register/retire/hold  
- Assign/unassign  
- Inspection pass/fail/void  
- Deficiency clear/defer  
- Maintenance complete  
- Certification revoke  
- Binder sign-off  
- QR bind/unbind  

Equipment also keeps operational history timelines on the asset (maintenance history, inspection history) for UX—not a replacement for Core Audit.

---

## 17. Data Ownership

### 17.1 Schema `equipment` Owns

- Assets, types, profiles, QR bindings, photo refs  
- Checklist definitions, inspections  
- Deficiencies, maintenance orders/history  
- Certification records  
- Binder templates & binder instances  
- Readiness projections, reporting projections  

### 17.2 References Only

| Ref | Owner |
| --- | --- |
| File bytes / photos | Core Files |
| Controlled manuals/letters | Documents |
| Signature packages | Signatures |
| Operator competency | Training |
| Project active status | Projects |
| Person active status | People |
| Lift plan case logic | Safety |

---

## 18. Integration With Other Modules

| Module | How it consumes Equipment |
| --- | --- |
| **Safety** | Lift/permit/asset refs; must call `GetAssetReadiness` before high-risk approve/use |
| **Projects** | Equipment requirements; dashboard counters from Equipment events |
| **People** | Operator/custodian identity on assignments/inspections |
| **Documents** | Cert/manual evidence versions linked from certs/binders |
| **Signatures** | Inspection and binder sign-offs |
| **Training** | Competency checks for operate/assign policies |
| **COR** | Inspection/cert/binder evidence provenance |
| **Notifications** | Due/failed/expiry fan-out |
| **Analytics** | Fleet KPIs from events |
| **Web/PWA** | Scan QR → readiness → pre-use → seal |

---

## 19. Anti-Patterns

1. Treating Safety site inspections as a substitute for equipment pre-use SoR  
2. Storing binder PDFs only on a shared drive reference without `CertificationRecord`/`BinderSection`  
3. Editing readiness manually without hold reason/audit  
4. Client-only reminders for periodic/cert expiry  
5. Duplicating Training completions inside Equipment  
6. Creating a second project ACL for who can use assets  
7. QR that invents assets on unknown scan  

---

## 20. Success Criteria

Equipment is correctly designed when:

1. Every in-use asset has stable identity, QR, and derived readiness.  
2. Pre-use and periodic inspections produce sealed, auditable proof.  
3. Tower and self-erect cranes cannot show Ready with incomplete binders.  
4. Safety lift plans trust `GetAssetReadiness` as the gate.  
5. Deficiencies and maintenance form a coherent history without becoming a finance CMMS.  
6. Expiry and overdue behavior is workflow-durable and notification-complete.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Principal Architecture | Initial Equipment domain architecture |

---

*End of Equipment Domain Architecture*
