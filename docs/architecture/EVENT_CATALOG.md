# Proven — Event-Driven Architecture & Event Catalog

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Event-Driven Architecture |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Platform / Domain Architecture |
| **Audience** | Backend, Workers, Analytics, Integrations |
| **Last updated** | 2026-08-03 |
| **Companion docs** | Module `*_DOMAIN.md` docs, [System Architecture](./SYSTEM_ARCHITECTURE.md), [Rust Backend](./RUST_BACKEND_ARCHITECTURE.md), [PostgreSQL Architecture](./POSTGRESQL_ARCHITECTURE.md) |

---

## 1. Purpose

This document defines **every integration/domain event** used by Proven: naming, payload shape, publisher, subscribers, retry, ordering, versioning, and lifecycle.

Transport: **PostgreSQL transactional outbox → NATS** (primary). Temporal signals are process orchestration, not this catalog—though workflows often **react to** or **cause** these events.

**Documentation only — no implementation.**

---

## 2. Event Architecture Overview

```text
Command succeeds in owning module
  → Domain events raised in-process
  → Persisted to platform.outbox_messages (same DB transaction)
  → Commit
  → Outbox publisher publishes to NATS
  → Consumer groups (Notifications, Analytics ingest, COR, projections, Go workers)
  → Idempotent handlers
```

### 2.1 Subject Naming

```text
proven.<module>.v<major>.<EventName>
```

Examples:

- `proven.people.v1.WorkerCreated`
- `proven.safety.v1.FLHASubmitted`
- `proven.signatures.v1.DocumentSigned`

### 2.2 Event Kinds

| Kind | Description |
| --- | --- |
| **Domain / integration event** | Business fact after commit (this catalog) |
| **Delivery / ops event** | Notification attempt outcomes (still versioned) |
| **Temporal signal** | Workflow-internal (out of band) |

---

## 3. Envelope (All Events)

Every event carries this envelope; type-specific data lives in `payload`.

| Field | Type | Description |
| --- | --- | --- |
| `event_id` | UUID | Unique id (idempotency key for consumers) |
| `event_name` | string | PascalCase past-tense name |
| `event_version` | semver string | Payload schema version (`1.0.0`) |
| `occurred_at` | datetime | When fact happened (domain time) |
| `published_at` | datetime | When outbox published (optional) |
| `tenant_id` | UUID | Tenant isolation |
| `actor` | object | `{ kind, principal_id?, user_id?, person_id?, system? }` |
| `correlation_id` | UUID | Request/workflow correlation |
| `causation_id` | UUID | Parent event or command id |
| `resource` | object | `{ type, id }` primary resource |
| `project_id` | UUID? | When project-scoped |
| `payload` | object | Event-specific fields |
| `schema` | string? | Optional schema URL/id |

**Rules:** No passwords, magic-link secrets, signature strokes, or medical note bodies. PHI minimized to coarse signals when required.

---

## 4. Versioning

| Rule | Detail |
| --- | --- |
| Name stability | Event names are forever within a major transport version |
| Payload evolution | **Additive only** within `event_version` major |
| Breaking payload | Bump `event_version` major; publish dual-write or new subject `v2` during migration |
| Consumers | Ignore unknown fields; tolerate missing new optional fields |
| Contracts | Schemas live under `contracts/events/` |

Alias note: product language may say `WorkerCreated`; canonical catalog uses names below (with aliases called out).

---

## 5. Ordering

| Scope | Guarantee |
| --- | --- |
| **Per aggregate partition key** | Outbox publisher preserves commit order for same `(tenant_id, resource.type, resource.id)` when using per-key sequencing |
| **Global tenant order** | **Not** guaranteed |
| **Cross-aggregate** | **Not** guaranteed—subscribers must not assume |
| **NATS** | At-least-once; consumers idempotent on `event_id` |

**Partition key recommendation:** `tenant_id + resource.type + resource.id` for causal streams (e.g., activity lifecycle).

Consumers that need strict follow-up use Temporal workflows or read model versions—not global bus order.

---

## 6. Retry Strategy

### 6.1 Publisher (Outbox → NATS)

| Failure | Action |
| --- | --- |
| Transient NATS | Exponential backoff; remain unpublished |
| Poison payload | Quarantine outbox row; alert; do not block partition forever—ops replay |

### 6.2 Subscribers

| Error class | Action |
| --- | --- |
| Transient | Nak/retry with backoff; max attempts then DLQ |
| Permanent / validation | Term to DLQ; alert |
| Handler bug | DLQ + fix + replay |

**Idempotency:** store processed `event_id` (or upsert by natural key).

**DLQ subject:** `proven.dlq.<module>.v1`

Critical compliance consumers (COR readiness, Training expiry projections) page on DLQ depth.

---

## 7. Event Lifecycle

```text
1. Raised     Domain aggregate / application service creates event in memory
2. Recorded   Written to outbox in same transaction as state change
3. Committed  DB commit makes event durable with business state
4. Published  Outbox worker emits to NATS subject
5. Delivered  Consumer group receives (at least once)
6. Handled    Idempotent projection / notify / ingest
7. Completed  Ack; mark processed
8. Retained   Outbox row retained short window; event may live in analytics lake
9. Archived   Outbox purged per retention; CH/lake keep facts as needed
```

**Void/correction:** never mutate past events—publish compensating events (`…Voided`, `…Revoked`).

---

## 8. Subscriber Legend

| Code | System |
| --- | --- |
| `NOTIF` | Notifications module (+ Go delivery workers) |
| `ANLY` | Analytics ingest (Go → ClickHouse) |
| `COR` | COR readiness / mappings |
| `PROJ` | Projects projections (dashboard) |
| `PEOP` | People projections (history, competency cards) |
| `SAFE` | Safety (rare; usually publisher) |
| `EQP` | Equipment projections |
| `DOC` | Documents |
| `TRN` | Training assignment sync / projections |
| `SIG` | Signatures (rare consumer) |
| `WF` | Workflow starters/signals |
| `ADMIN` | Admin health / audit views |
| `CORE` | Core cache invalidation (authz/flags) |
| `AUDIT` | Optional fan-in to analytics of audit (Core audit is sync write, not bus-dependent) |

---

## 9. Canonical Catalog by Module

Payload columns list **payload fields** (envelope separate). Types are logical.

---

### 9.1 Core (`proven.core.v1.*`)

| Event Name | Payload (key fields) | Publisher | Subscribers |
| --- | --- | --- | --- |
| `TenantProvisioned` | `tenant_id`, `region_code`, `slug` | Core | NOTIF, ANLY, ADMIN, WF |
| `TenantSuspended` | `reason?` | Core | NOTIF, ANLY, WF, all caches |
| `TenantReactivated` | — | Core | NOTIF, ANLY |
| `TenantClosed` | — | Core | ANLY, ADMIN |
| `CompanyRegistered` | `company_id`, `company_type`, `name` | Core | PROJ, PEOP, ANLY |
| `CompanyUpdated` | `company_id`, `changed_fields[]` | Core | PROJ, PEOP, ANLY |
| `CompanyDeactivated` | `company_id` | Core | PROJ, PEOP |
| `OrgUnitCreated` | `org_unit_id`, `parent_id?` | Core | ANLY |
| `OrgUnitMoved` | `org_unit_id`, `parent_id` | Core | ANLY |
| `OrgUnitArchived` | `org_unit_id` | Core | ANLY |
| `UserInvited` | `user_id`, `email`, `person_id?` | Core | NOTIF, PEOP, ANLY |
| `UserActivated` | `user_id` | Core | NOTIF, CORE |
| `UserDeactivated` | `user_id` | Core | NOTIF, CORE, WF |
| `UserLocked` / `UserUnlocked` | `user_id`, `reason?` | Core | NOTIF, AUDIT path |
| `UserLinkedToPerson` | `user_id`, `person_id` | Core | PEOP, NOTIF |
| `ExternalIdentityLinked` | `user_id`, `provider` | Core | ADMIN |
| `SessionEstablished` | `session_id`, `user_id` | Core | ANLY (optional security) |
| `SessionRevoked` | `session_id`, `user_id` | Core | CORE cache |
| `RoleDefinitionChanged` | `role_id` | Core | CORE cache, ADMIN |
| `AccessGranted` | `user_id`, `role_id`, `scope` | Core | CORE cache, NOTIF, ANLY |
| `AccessRevoked` | `user_id`, `grant_id`, `scope` | Core | CORE cache, NOTIF |
| `ProjectMembershipGranted` | `project_id`, `person_id`, `user_id?`, `roles[]` | Core | TRN, PROJ, PEOP, NOTIF, SAFE, ANLY, WF |
| `ProjectMembershipUpdated` | `membership_id`, `roles[]` | Core | TRN, PROJ, PEOP, ANLY |
| `ProjectMembershipRevoked` | `project_id`, `person_id` | Core | TRN, PROJ, PEOP, NOTIF, ANLY |
| `TeamCreated` | `team_id`, `project_id?` | Core | PROJ, SAFE, ANLY |
| `TeamMemberAdded` / `TeamMemberRemoved` | `team_id`, `person_id` | Core | PROJ, SAFE, PEOP |
| `TeamArchived` | `team_id` | Core | PROJ |
| `FileUploadIntentCreated` | `file_object_id`, `content_type`, `size` | Core | media workers |
| `FileObjectAvailable` | `file_object_id`, `checksum` | Core | DOC, SAFE, EQP, SIG, ANLY |
| `FileObjectQuarantined` | `file_object_id`, `reason` | Core | NOTIF, ADMIN |
| `FileObjectDeleted` | `file_object_id` | Core | DOC, SAFE, EQP |
| `SettingsChanged` | `scope`, `keys[]` | Core | NOTIF, modules as needed |
| `FeatureFlagChanged` | `flag_key`, `enabled` | Core | CORE cache, all optional |
| `LicenseActivated` / `LicenseUpdated` | `license_id`, `entitlements` | Core | ADMIN, NOTIF, ANLY |
| `LicenseExpiring` / `LicenseExpired` / `LicenseSuspended` | `license_id` | Core | NOTIF, ADMIN, WF |
| `SeatAllocated` / `SeatReleased` | `seat_type`, `user_id?` | Core | ADMIN, ANLY |
| `ModuleEntitlementChanged` | `module_key`, `enabled` | Core | ADMIN, NOTIF, ANLY |
| `AuditEntryAppended` | `audit_entry_id`, `action`, `resource` *(optional bus)* | Core | ANLY (optional); prefer sync audit SoR |

---

### 9.2 People / Workers (`proven.people.v1.*`)

| Event Name | Payload | Publisher | Subscribers |
| --- | --- | --- | --- |
| `WorkerCreated` **(alias `PersonRegistered`)** | `person_id`, `name`, `status` | People | CORE, NOTIF, ANLY, TRN, PROJ |
| `WorkerUpdated` (`PersonUpdated`) | `person_id`, `changed_fields[]` | People | ANLY, PROJ |
| `WorkerActivated` / `WorkerDeactivated` | `person_id` | People | TRN, PROJ, SAFE, EQP, NOTIF, ANLY |
| `WorkerArchived` | `person_id` | People | ANLY |
| `WorkforceRoleAssigned` / `WorkforceRoleRemoved` | `person_id`, `workforce_role` | People | TRN, ANLY |
| `TradeAssigned` / `TradeRemoved` | `person_id`, `trade_code` | People | TRN, ANLY, WF |
| `EmergencyContactAdded` / `Updated` / `Removed` | `person_id`, `contact_id` (no full PII on bus) | People | — (minimize) |
| `MedicalRestrictionRecorded` / `Updated` / `Cleared` | `person_id`, `restriction_id`, `fit_signal` | People | SAFE, PROJ, NOTIF (coarse) |
| `FitForWorkSignalChanged` | `person_id`, `fit_signal` | People | SAFE, EQP, PROJ, ANLY |
| `EmploymentStarted` / `Updated` / `Ended` | `person_id`, `company_id`, `employment_id` | People | ANLY, PROJ |
| `ContractorEngagementStarted` / `Ended` | `person_id`, `company_id` | People | ANLY, PROJ |
| `AvailabilityUpdated` | `person_id`, `range` | People | PROJ, NOTIF optional |
| `AttendanceRecorded` / `Corrected` / `Voided` | `person_id`, `project_id?`, `work_date`, `status` | People | ANLY, PROJ |
| `CertificationProfileEntryAdded` / `Removed` | `person_id`, `refs` | People | ANLY |
| `CompetencyProfileRebuilt` | `person_id` | People | — |
| `PersonAssignmentViewUpdated` | `person_id`, `project_id` | People | — |
| `PersonSignatureHistoryAppended` | `person_id`, `signature_package_id` | People | — |
| `PersonHistoryAppended` | `person_id`, `entry_type` | People | — |

---

### 9.3 Projects (`proven.projects.v1.*`)

| Event Name | Payload | Publisher | Subscribers |
| --- | --- | --- | --- |
| `ProjectCreated` | `project_id`, `code`, `name`, `status` | Projects | NOTIF, ANLY, COR, PEOP, TRN, WF |
| `ProjectUpdated` | `project_id`, `changed_fields[]` | Projects | ANLY, NOTIF |
| `ProjectActivated` | `project_id` | Projects | SAFE, EQP, TRN, NOTIF, ANLY, COR, WF |
| `ProjectPutOnHold` / `ProjectResumed` | `project_id` | Projects | SAFE, EQP, TRN, NOTIF, ANLY |
| `ProjectClosed` / `ProjectArchived` | `project_id` | Projects | SAFE, EQP, TRN, NOTIF, ANLY, COR |
| `ProjectReopened` | `project_id` | Projects | ANLY, NOTIF |
| `ProjectParticipantAdded` / `Updated` / `Removed` | `project_id`, `company_id`, `participation_role` | Projects | ANLY, COR, NOTIF |
| `ProjectPrimeAssigned` / `ProjectPrimeChanged` | `project_id`, `company_id` | Projects | ANLY, NOTIF, COR |
| `ProjectClientAssigned` | `project_id`, `company_id` | Projects | ANLY |
| `ProjectLocationSet` | `project_id` | Projects | ANLY |
| `ProjectAreaAdded` / `Updated` / `Deactivated` | `project_id`, `area_id` | Projects | SAFE, EQP, ANLY |
| `RequiredControlDefined` / `Updated` / `Removed` | `project_id`, `control_ref` | Projects | SAFE, TRN, DOC, EQP, ANLY |
| `ProjectFormBindingAdded` / `Removed` | `project_id`, `form_type_id` | Projects | SAFE, ANLY |
| `ProjectDocumentLinked` / `Unlinked` | `project_id`, `document_id`, `purpose` | Projects | DOC, ANLY, COR |
| `EquipmentRequirementDefined` / `Removed` | `project_id`, `asset_class` | Projects | EQP, ANLY |
| `ProjectTeamLinked` / `Unlinked` | `project_id`, `team_id` | Projects | SAFE, PEOP |
| `ProjectSettingsChanged` | `project_id`, `keys[]` | Projects | SAFE, NOTIF |
| `ProjectTemplateCreated` / `Published` / `Retired` | `template_id` | Projects | ADMIN, ANLY |
| `ProjectCreatedFromTemplate` | `project_id`, `template_id` | Projects | ANLY, WF |
| `ProjectDashboardRebuilt` | `project_id` | Projects | — |
| `ProjectProofHealthChanged` | `project_id`, `proof_health` | Projects | ANLY, COR, NOTIF |

---

### 9.4 Safety (`proven.safety.v1.*`)

| Event Name | Payload | Publisher | Subscribers |
| --- | --- | --- | --- |
| `FLHASubmitted` *(also `SafetyActivitySubmitted` with type=flha)* | `activity_id`, `project_id`, `activity_type`, `status` | Safety | NOTIF, ANLY, COR, PROJ, WF, SIG |
| `SafetyActivityOpened` / `Updated` | `activity_id`, `project_id`, `activity_type` | Safety | ANLY, PROJ |
| `SafetyActivitySubmitted` | same + `submitted_at` | Safety | NOTIF, ANLY, COR, PROJ, WF |
| `SafetyActivityReviewRequested` / `Reviewed` | `activity_id`, `reviewer_person_id?` | Safety | NOTIF, ANLY, WF |
| `SafetyActivityClosed` / `Voided` | `activity_id`, `reason?` | Safety | ANLY, COR, PROJ, NOTIF |
| `SafetyActivitySignatureRequested` | `activity_id`, `signature_package_id` | Safety | SIG, NOTIF |
| `SafetyActivitySealed` | `activity_id`, `signature_package_id` | Safety | ANLY, COR, PROJ, NOTIF |
| `AttendanceRecorded` | `activity_id`, `person_ids[]` count only preferred | Safety | ANLY |
| `WeatherSnapshotRecorded` | `activity_id` | Safety | ANLY optional |
| `AttachmentAdded` | `activity_id`, `file_object_id` | Safety | ANLY |
| `ProcedureAcknowledged` | `activity_id`, `document_version_id` | Safety | DOC, COR, ANLY |
| `ToolboxTalkCompleted` | `activity_id`, `project_id`, `sealed` | Safety | ANLY, COR, PROJ, NOTIF |
| `HazardLibraryItemDefined` / `Updated` / `Retired` | `item_id` | Safety | — |
| `ControlLibraryItemDefined` / `Updated` / `Retired` | `item_id` | Safety | — |
| `RiskMatrixPublished` / `Retired` | `matrix_id` | Safety | — |
| `CorrectiveActionCreated` (`CorrectiveActionOpened`) | `corrective_action_id`, `project_id`, `source_ref`, `due_at`, `severity?` | Safety | NOTIF, ANLY, COR, PROJ, WF |
| `CorrectiveActionAssigned` / `Updated` | `corrective_action_id`, `owner_person_id` | Safety | NOTIF, ANLY, WF |
| `CorrectiveActionOverdue` | `corrective_action_id`, `due_at` | Safety | NOTIF, ANLY, COR, PROJ, WF |
| `CorrectiveActionCompleted` / `Verified` / `Closed` / `Cancelled` | `corrective_action_id` | Safety | NOTIF, ANLY, COR, PROJ |
| `NearMissReported` | `activity_or_case_id`, `project_id` | Safety | NOTIF, ANLY, COR |
| `IncidentReported` (`IncidentCaseOpened`) | `incident_case_id`, `project_id`, `severity` | Safety | NOTIF, ANLY, COR, PROJ, WF |
| `IncidentInvestigationUpdated` / `IncidentCaseClosed` | `incident_case_id` | Safety | NOTIF, ANLY, COR |
| `CriticalRiskRaised` | `resource_ref`, `project_id` | Safety | NOTIF (forced), ANLY, COR, WF |
| `SafetyBulletinPublished` / `Acknowledged` / `Closed` | `bulletin_id`, `project_id?` | Safety | NOTIF, ANLY, COR |
| `PermitRequested` / `Issued` / `Suspended` / `Closed` | `permit_case_id`, `project_id` | Safety | NOTIF, ANLY, EQP, COR |
| `LiftPlanCreated` / `Approved` / `Completed` / `Voided` | `lift_plan_id`, `project_id`, `asset_id?` | Safety | NOTIF, ANLY, EQP, COR |
| `DailyLogOpened` / `Updated` / `Closed` | `daily_log_id`, `project_id`, `work_date` | Safety | ANLY, PROJ |
| `SafetyProcedureBound` | `project_id`, `document_version_id` | Safety | DOC, COR |

---

### 9.5 Equipment (`proven.equipment.v1.*`)

| Event Name | Payload | Publisher | Subscribers |
| --- | --- | --- | --- |
| `AssetRegistered` / `Updated` | `asset_id`, `class`, `asset_tag` | Equipment | ANLY, PROJ |
| `AssetAssignedToProject` / `Unassigned` | `asset_id`, `project_id` | Equipment | PROJ, ANLY, NOTIF, SAFE |
| `AssetAssignedToPerson` | `asset_id`, `person_id` | Equipment | PEOP, ANLY |
| `AssetTakenOutOfService` / `ReturnedToService` / `Retired` | `asset_id`, `reason?` | Equipment | NOTIF, ANLY, PROJ, SAFE |
| `AssetQrBound` | `asset_id`, `qr_code_id` | Equipment | ANLY |
| `AssetPhotoAdded` | `asset_id`, `file_object_id` | Equipment | ANLY |
| `AssetReadinessChanged` | `asset_id`, `readiness`, `reasons[]` | Equipment | SAFE, PROJ, NOTIF, ANLY, COR |
| `InspectionStarted` / `Submitted` | `inspection_id`, `asset_id`, `kind` | Equipment | ANLY, WF |
| `EquipmentInspectionCompleted` *(Passed or Failed)* | `inspection_id`, `asset_id`, `kind`, `result` | Equipment | NOTIF, ANLY, COR, PROJ, SAFE, SIG |
| `InspectionPassed` / `InspectionFailed` / `Voided` | same | Equipment | same |
| `PreUseInspectionDue` / `PeriodicInspectionDue` / `Overdue` | `asset_id`, `due_at` | Equipment/WF | NOTIF, ANLY, PROJ |
| `DeficiencyOpened` / `Updated` / `Cleared` / `Deferred` | `deficiency_id`, `asset_id`, `severity` | Equipment | NOTIF, ANLY, COR, PROJ, WF |
| `MaintenanceOrderCreated` / `Completed` | `maintenance_order_id`, `asset_id` | Equipment | NOTIF, ANLY, COR |
| `MaintenanceHistoryAppended` | `asset_id` | Equipment | ANLY |
| `CertificationRecorded` / `Expiring` / `Expired` / `Revoked` | `certification_id`, `asset_id`, `expires_at?` | Equipment | NOTIF, ANLY, COR, PROJ, WF |
| `BinderCreated` / `SectionCompleted` / `SectionExpired` | `binder_id`, `asset_id` | Equipment | NOTIF, ANLY, COR |
| `BinderCompletenessChanged` | `binder_id`, `completeness` | Equipment | ANLY, PROJ, SAFE |
| `BinderSignedOff` | `binder_id`, `signature_package_id` | Equipment | SIG, COR, ANLY |

---

### 9.6 Documents (`proven.documents.v1.*`)

| Event Name | Payload | Publisher | Subscribers |
| --- | --- | --- | --- |
| `DocumentCreated` / `Updated` | `document_id`, `category` | Documents | ANLY, COR |
| `DocumentArchived` / `Retired` / `Restored` | `document_id` | Documents | ANLY, COR, NOTIF |
| `DocumentVersionCreated` / `Updated` | `document_version_id`, `document_id`, `state` | Documents | ANLY |
| `DocumentVersionSubmittedForReview` / `ForApproval` | `document_version_id` | Documents | NOTIF, WF, ANLY |
| `DocumentVersionApproved` / `Rejected` | `document_version_id` | Documents | NOTIF, ANLY, WF |
| `DocumentVersionPublished` | `document_version_id`, `document_id`, `effective_from` | Documents | NOTIF, SAFE, TRN, EQP, COR, ANLY, PROJ, WF |
| `DocumentVersionSuperseded` / `Withdrawn` | `document_version_id` | Documents | SAFE, TRN, COR, ANLY, NOTIF |
| `DocumentAssignmentCreated` / `Cancelled` | `assignment_id`, `document_version_id` | Documents | NOTIF, ANLY, WF |
| `AcknowledgementRequested` | `ack_request_id`, `person_id?`, `document_version_id` | Documents | NOTIF, ANLY, WF |
| `DocumentAcknowledged` | `document_version_id`, `person_id`, `signature_package_id?` | Documents | TRN, SAFE, COR, ANLY, PEOP, PROJ |
| `DocumentAcknowledgementOverdue` | `ack_request_id` | Documents | NOTIF, ANLY, WF |
| `DocumentSignatureRequested` | `document_version_id`, `signature_package_id` | Documents | SIG, NOTIF |
| `DocumentSigned` *(ack/sign completed with seal)* | `document_version_id`, `signature_package_id`, `person_id?` | Documents or Signatures bridge | DOC consumers, COR, ANLY, NOTIF |
| `GuestSignLinkIssued` | `link_id`, `package_id` (no secret) | Documents/SIG | NOTIF |
| `DocumentQrSignTargetIssued` / `Completed` | `qr_id`, `document_version_id` | Documents | SIG, ANLY |
| `DistributionIssued` | `distribution_id` | Documents | NOTIF, ANLY |
| `RetentionPolicyApplied` | `document_id`, `policy_id` | Documents | ADMIN |
| `LegalHoldApplied` / `Released` | `document_id` | Documents | ADMIN, NOTIF |
| `DocumentDisposalEligible` / `Disposed` | `document_id` | Documents | ADMIN, ANLY |
| `DocumentSearchProjectionUpdated` | `document_version_id` | Documents | — |

---

### 9.7 Training (`proven.training.v1.*`)

| Event Name | Payload | Publisher | Subscribers |
| --- | --- | --- | --- |
| `TrainingCourseDefined` / `Updated` / `Retired` | `course_id`, `kind` | Training | ANLY |
| `CompetencyDefinitionDefined` / `Updated` / `Retired` | `competency_id` | Training | ANLY |
| `EvaluationDefinitionPublished` | `evaluation_definition_id` | Training | ANLY |
| `ToolboxLibraryItemDefined` / `Updated` / `Retired` | `item_id` | Training | SAFE |
| `TrainingRequirementAssigned` / `Removed` | `requirement_id`, `scope` | Training | NOTIF, ANLY, WF, PEOP |
| `TrainingAssignmentCreated` / `Updated` / `Completed` / `Cancelled` | `assignment_id`, `person_id`, `course_id` | Training | NOTIF, ANLY, PEOP, WF |
| `TrainingAssignmentOverdue` | `assignment_id`, `person_id` | Training | NOTIF, ANLY, PROJ |
| `EvaluationAttemptStarted` / `Submitted` / `Passed` / `Failed` | `attempt_id`, `person_id` | Training | NOTIF, ANLY, SIG |
| `TrainingCompletionRecorded` | `completion_id`, `person_id`, `course_id`, `valid_from`, `valid_to?` | Training | PEOP, NOTIF, ANLY, COR, SAFE, EQP, PROJ |
| `TrainingExpiring` (`TrainingCompletionExpiring`) | `completion_id`, `person_id`, `valid_to` | Training | NOTIF, ANLY, PEOP, WF |
| `TrainingExpired` (`TrainingCompletionExpired`) | `completion_id`, `person_id`, `course_id` | Training | NOTIF, ANLY, COR, PEOP, SAFE, EQP, PROJ, WF |
| `TrainingCompletionRevoked` | `completion_id`, `reason` | Training | same as expired |
| `TrainingWaiverGranted` / `Expired` | `waiver_id`, `person_id` | Training | NOTIF, ANLY, COR, PEOP |
| `RenewalCaseOpened` / `Completed` / `Overdue` | `renewal_case_id`, `person_id` | Training | NOTIF, ANLY, WF |
| `CompetencyGapDetected` / `Resolved` | `person_id`, `competency_id`, `project_id?` | Training | NOTIF, ANLY, COR, PROJ, SAFE, EQP |
| `TrainingMatrixRebuilt` | `tenant_id`, `project_id?` | Training | — |

---

### 9.8 Signatures (`proven.signatures.v1.*`)

| Event Name | Payload | Publisher | Subscribers |
| --- | --- | --- | --- |
| `SigningPolicyChanged` | `policy_id`, `process_type` | Signatures | DOC, SAFE, TRN, EQP |
| `SignaturePackageCreated` | `signature_package_id`, `subject`, `process_type` | Signatures | NOTIF, ANLY, subject module, WF |
| `SignerAssigned` / `Reassigned` | `package_id`, `slot_id`, `assignee` | Signatures | NOTIF |
| `MagicLinkIssued` / `Redeemed` / `Revoked` / `Expired` | `link_id`, `package_id` (no secret) | Signatures | NOTIF, ANLY security optional |
| `QrSignSessionOpened` / `Completed` / `Expired` | `session_id`, `package_id` | Signatures | ANLY |
| `IdentityAssuranceCaptured` | `package_id`, `slot_id`, `assurance_level` | Signatures | ANLY |
| `SignatureCaptured` | `package_id`, `slot_id` | Signatures | ANLY, subject module |
| `SignerDeclined` | `package_id`, `slot_id`, `reason?` | Signatures | NOTIF, subject, WF |
| `SignaturePackagePartiallyCompleted` | `package_id`, `sealed_count`, `required_count` | Signatures | NOTIF, subject, ANLY |
| `SignaturePackageCompleted` | `package_id`, `subject` | Signatures | DOC, SAFE, TRN, EQP, COR, PEOP, NOTIF, ANLY, PROJ |
| `DocumentSigned` *(when subject is document version)* | `package_id`, `document_version_id` | Signatures | DOC, COR, ANLY, NOTIF |
| `SignaturePackageVoided` / `Expired` | `package_id`, `reason?` | Signatures | subject, NOTIF, ANLY, COR |
| `EvidenceCertificateGenerated` | `package_id`, `certificate_id`, `file_object_id` | Signatures | COR, ANLY, NOTIF |
| `DocumentVersionValidationFailed` | `package_id`, `document_version_id` | Signatures | DOC, NOTIF, WF |
| `SignatureReminderSent` | `package_id`, `slot_id` | Signatures/NOTIF | ANLY optional |

---

### 9.9 COR (`proven.cor.v1.*`)

| Event Name | Payload | Publisher | Subscribers |
| --- | --- | --- | --- |
| `AuditFrameworkPublished` / `Retired` | `framework_id`, `family`, `version` | COR | ANLY, ADMIN |
| `ReadinessProfileInitialized` | `profile_id`, `framework_id`, `subject` | COR | ANLY |
| `EvidenceLinkedToElement` / `Unlinked` | `mapping_id`, `element_id`, `provenance` | COR | ANLY |
| `ReadinessRecalculated` | `profile_id`, `score`, `coverage_summary` | COR | ANLY, NOTIF, PROJ, ADMIN |
| `GapOpened` / `Closed` / `Assigned` | `gap_id`, `element_id`, `owner?`, `due_at?` | COR | NOTIF, ANLY, WF |
| `AuditPlanCreated` / `Updated` | `plan_id` | COR | NOTIF, ANLY |
| `AuditEngagementOpened` / `StatusChanged` / `Closed` | `engagement_id`, `type`, `status` | COR | NOTIF, ANLY, WF |
| `InterviewRecorded` | `interview_id`, `engagement_id` | COR | ANLY |
| `ObservationRecorded` | `observation_id`, `engagement_id` | COR | ANLY |
| `AuditFindingOpened` / `Updated` / `Closed` | `finding_id`, `severity` | COR | NOTIF, ANLY, WF |
| `AuditCorrectiveActionOpened` / `Completed` / `Verified` | `audit_ca_id` | COR | NOTIF, ANLY, SAFE optional |
| `ScorecardCalculated` | `engagement_id`, `overall_score` | COR | ANLY, NOTIF |
| `HistoricalAuditRecorded` | `engagement_id` | COR | ANLY |
| `EvidencePackageRequested` / `Generated` / `Failed` | `package_id`, `engagement_id?` | COR | NOTIF, ANLY, WF, Go report workers |
| `AuditReportGenerated` / `Finalized` | `report_id` | COR | NOTIF, ANLY |
| `CorDashboardRebuilt` / `CorAnalyticsProjectionUpdated` | `subject?` | COR | — |

---

### 9.10 Notifications (`proven.notifications.v1.*`)

| Event Name | Payload | Publisher | Subscribers |
| --- | --- | --- | --- |
| `NotificationCreated` / `Queued` / `Dispatched` | `notification_id`, `priority`, `template_code` | Notifications | ANLY |
| `NotificationSent` *(channel send succeeded; prefer `DeliveryAttemptSucceeded` + `NotificationDelivered`)* | `notification_id`, `channel`, `provider_message_id?` | Notifications / workers | ANLY |
| `NotificationDelivered` / `PartiallyDelivered` / `Failed` / `Cancelled` | `notification_id` | Notifications | ANLY |
| `NotificationRead` / `Dismissed` | `notification_id` | Notifications | ANLY |
| `NotificationEscalated` | `notification_id`, `step` | Notifications | ANLY |
| `DeliveryAttemptStarted` / `Succeeded` / `Failed` | `job_id`, `channel`, `error_class?` | Notifications | ANLY |
| `DeliveryDeadLetter` | `job_id` | Notifications | ADMIN, ANLY |
| `DigestBatchCreated` / `Sent` | `digest_batch_id` | Notifications | ANLY |
| `NotificationTemplatePublished` | `template_code` | Notifications | — |
| `DeliveryRuleChanged` / `PreferenceUpdated` / `SubscriptionChanged` | ids | Notifications | — |
| `EscalationPolicyChanged` / `DigestScheduleChanged` | ids | Notifications | — |
| `ChannelConnectorConfigured` | `connector_id`, `channel` | Notifications | ADMIN |

---

### 9.11 Analytics (`proven.analytics.v1.*`)

| Event Name | Payload | Publisher | Subscribers |
| --- | --- | --- | --- |
| `MetricDefinitionPublished` | `metric_key` | Analytics | — |
| `DashboardDefinitionPublished` | `dashboard_id` | Analytics | — |
| `ReportDefinitionPublished` | `report_id` | Analytics | — |
| `AnalyticsSubscriptionChanged` | `subscription_id` | Analytics | NOTIF |
| `ExportJobCompleted` / `Failed` | `job_id`, `file_object_id?` | Analytics | NOTIF, ANLY |
| `AnalyticsProjectionRebuilt` | `scope` | Analytics | ADMIN |
| `ScheduledReportDispatched` | `subscription_id`, `job_id` | Analytics | NOTIF |

---

### 9.12 Workflows (`proven.workflows.v1.*`)

| Event Name | Payload | Publisher | Subscribers |
| --- | --- | --- | --- |
| `WorkflowStarted` | `instance_id`, `definition_id`, `subject` | Workflows | NOTIF optional, ANLY |
| `WorkflowSignaled` | `instance_id`, `signal_name` | Workflows | ANLY |
| `WorkflowCompleted` / `Failed` / `Cancelled` | `instance_id`, `status` | Workflows | NOTIF, ANLY, subject module |
| `EscalationTriggered` | `instance_id`, `policy_step` | Workflows | NOTIF, ANLY |

---

### 9.13 Admin (`proven.admin.v1.*`)

| Event Name | Payload | Publisher | Subscribers |
| --- | --- | --- | --- |
| `TenantBrandingUpdated` | `tenant_id` | Admin | Web cache, ANLY |
| `AdminConsoleSettingsChanged` | — | Admin | — |
| `ApiClientCreated` / `Updated` | `api_client_id` | Admin | ANLY |
| `ApiKeyIssued` / `Rotated` / `Revoked` / `Expired` | `api_key_id` (no secret) | Admin | NOTIF, ANLY |
| `IntegrationRegistered` / `Connected` / `Degraded` / `Disconnected` | `integration_id` | Admin | NOTIF, ANLY, WF |
| `BuilderDraftCreated` / `Updated` / `Discarded` | `draft_id`, `kind` | Admin | — |
| `BuilderPublishRequested` / `Succeeded` / `Failed` | `draft_id`, `target` | Admin | NOTIF, owning module, ANLY |
| `AdminDashboardDefinitionChanged` | — | Admin | — |
| `SystemHealthSnapshotTaken` | `status` | Admin | NOTIF if unhealthy |
| `BillingAccountUpdated` | `billing_account_id` | Admin (future) | NOTIF, ANLY |

---

## 10. Example Event Deep Dives (User Samples)

### 10.1 `WorkerCreated`

| Aspect | Spec |
| --- | --- |
| **Name** | `WorkerCreated` (alias of `PersonRegistered`) |
| **Payload** | `person_id`, `display_name`, `status`, `company_id?`, `workforce_roles[]?` |
| **Publisher** | People |
| **Subscribers** | Notifications (welcome/setup), Analytics, Training (requirement sync trigger), Projects/People projections |
| **Retry** | Standard subscriber idempotent on `event_id` |
| **Ordering** | Per `person_id` |
| **Version** | `1.0.0` |
| **Lifecycle** | Outbox with `people.persons` insert |

### 10.2 `ProjectCreated`

| Aspect | Spec |
| --- | --- |
| **Payload** | `project_id`, `code`, `name`, `status`, `region_code?`, `template_id?` |
| **Publisher** | Projects |
| **Subscribers** | NOTIF, ANLY, COR, TRN, WF (onboarding), PEOP |
| **Ordering** | Per `project_id` |

### 10.3 `FLHASubmitted`

| Aspect | Spec |
| --- | --- |
| **Payload** | `activity_id`, `project_id`, `activity_type=flha`, `submitted_by_person_id`, `risk_rating?` |
| **Publisher** | Safety |
| **Subscribers** | NOTIF (reviewers), WF (review workflow), ANLY, COR, PROJ dashboard, SIG if signatures requested next |
| **Ordering** | Per `activity_id` after `SafetyActivityOpened` |

### 10.4 `DocumentSigned`

| Aspect | Spec |
| --- | --- |
| **Payload** | `signature_package_id`, `document_id`, `document_version_id`, `signer_person_id?`, `assurance_level` |
| **Publisher** | Signatures (preferred) when package completes for document subject; Documents may emit `DocumentAcknowledged` companion |
| **Subscribers** | Documents ack state, COR, ANLY, NOTIF, PEOP history |
| **Ordering** | Per `signature_package_id` / `document_version_id` |

### 10.5 `EquipmentInspectionCompleted`

| Aspect | Spec |
| --- | --- |
| **Payload** | `inspection_id`, `asset_id`, `kind`, `result` (`passed`\|`failed`), `project_id?` |
| **Publisher** | Equipment |
| **Subscribers** | Readiness recompute path, NOTIF on fail, ANLY, COR, PROJ, SAFE lift gates |
| **Ordering** | Per `asset_id` with readiness events |

### 10.6 `CorrectiveActionCreated`

| Aspect | Spec |
| --- | --- |
| **Payload** | `corrective_action_id`, `project_id`, `source_type`, `source_id`, `due_at`, `severity?` |
| **Publisher** | Safety |
| **Subscribers** | NOTIF, WF SLA, ANLY, COR, PROJ |
| **Ordering** | Per `corrective_action_id` |

### 10.7 `IncidentReported`

| Aspect | Spec |
| --- | --- |
| **Payload** | `incident_case_id`, `project_id`, `severity`, `reported_by_person_id` |
| **Publisher** | Safety |
| **Subscribers** | NOTIF (critical policy), WF investigation, ANLY, COR, PROJ |
| **Ordering** | Per `incident_case_id` |

### 10.8 `TrainingExpired`

| Aspect | Spec |
| --- | --- |
| **Payload** | `completion_id`, `person_id`, `course_id`, `competency_ids[]?`, `expired_at` |
| **Publisher** | Training (from expiry workflow) |
| **Subscribers** | NOTIF, CompetencyGapDetected path, ANLY, COR, SAFE/EQP eligibility consumers, PEOP, PROJ |
| **Ordering** | Per `person_id` + `completion_id` |

### 10.9 `NotificationSent`

| Aspect | Spec |
| --- | --- |
| **Canonical** | Prefer `DeliveryAttemptSucceeded` + `NotificationDelivered` |
| **Alias payload** | `notification_id`, `channel`, `recipient_user_id`, `provider_message_id?` |
| **Publisher** | Notifications (after worker callback) |
| **Subscribers** | ANLY delivery metrics |
| **Ordering** | Per `notification_id` |

---

## 11. Consumer Matrix (High Fan-In Events)

| Event | NOTIF | ANLY | COR | PROJ | TRN | PEOP | WF |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `ProjectMembershipGranted` | ✓ | ✓ | | ✓ | ✓ | ✓ | ✓ |
| `ProjectActivated` | ✓ | ✓ | ✓ | | ✓ | | ✓ |
| `FLHASubmitted` / activity submitted | ✓ | ✓ | ✓ | ✓ | | | ✓ |
| `CorrectiveActionOverdue` | ✓ | ✓ | ✓ | ✓ | | | ✓ |
| `AssetReadinessChanged` | ✓ | ✓ | ✓ | ✓ | | | |
| `TrainingExpired` | ✓ | ✓ | ✓ | ✓ | | ✓ | ✓ |
| `DocumentVersionPublished` | ✓ | ✓ | ✓ | ✓ | ✓ | | ✓ |
| `SignaturePackageCompleted` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | |
| `ReadinessRecalculated` | ✓ | ✓ | | ✓ | | | |

---

## 12. Contracts & Governance

1. New events require catalog entry + `contracts/events/<module>/<EventName>.v1.json` (or AsyncAPI).  
2. Publishers own schema; subscribers are conformist.  
3. No PII/PHI expansion without privacy review.  
4. Load tests must include outbox lag SLOs.  
5. Dual-publish during renames; never silent rename.

---

## 13. Success Criteria

Event architecture succeeds when:

1. Every state change that others must know has a named, versioned event.  
2. Outbox guarantees no lost publishes after commit.  
3. Consumers are idempotent under at-least-once delivery.  
4. Ordering expectations are explicit per aggregate—not mythical global order.  
5. Analytics, COR, Notifications, and projections stay decoupled from module internals.  
6. Compensating events, not edits, correct history.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Event-Driven Architecture | Complete Proven event catalog & EDA standards |

---

*End of Event-Driven Architecture*
