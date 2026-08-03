# Proven — Domain-Driven Design Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Domain Architecture (DDD) |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Architecture / Domain Engineering |
| **Audience** | Engineering, Product, Security |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [PRD](../PRD.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document defines the **complete business domain model** for Proven as a Construction Compliance Operating System.

It establishes:

- Bounded contexts and their strategic classification
- Domain ownership and modular monolith packaging
- Allowed interactions (public interfaces, domain events, Temporal workflows)
- Aggregate roots, entities, value objects, and domain events per context
- Evolution rules so every module can change independently without breaking the platform

**This document contains no application code.** It is the source of truth for domain boundaries.

---

## 2. Strategic Design Overview

### 2.1 Architectural Style

Proven is a **modular monolith**:

- One deployable backend (initially), one product surface.
- Many **independently owned modules**, each aligned to a bounded context.
- Modules **never** reach into another module’s internals (tables, private types, internal services).
- Modules integrate only through:
  1. **Public application interfaces** (synchronous queries/commands across a published API of the owning module)
  2. **Domain / integration events** (asynchronous facts published after successful commits)
  3. **Temporal workflows** (durable multi-step business processes that call module activities)

### 2.2 Domain Classification

| Classification | Meaning in Proven | Contexts |
| --- | --- | --- |
| **Core domains** | Differentiating Compliance OS capability | Safety, Training & Competency, COR Audit, Digital Evidence (Signatures), Equipment Compliance |
| **Supporting domains** | Necessary to run the OS; not the primary differentiator alone | Projects, Workforce (People), Document Control, Notifications |
| **Generic domains** | Commodity / platform capabilities adapted to Proven | Identity & Access, Tenancy & Administration, Workflow Orchestration, Analytics & Insights, Platform Audit |

> Core vs supporting can shift with market strategy. Boundaries remain stable even if strategic classification changes.

### 2.3 Ubiquitous Language (Platform-Level)

| Term | Meaning |
| --- | --- |
| **Tenant** | A customer organization boundary in the platform |
| **Company** | A legal/operating company within or related to a tenant (prime, sub, equipment firm, etc.) |
| **Project** | A construction undertaking that scopes people, equipment, documents, and compliance work |
| **Participant** | A company or person engaged on a project under a defined participation role |
| **Worker** | A person performing field work; mobile-first actor |
| **Eligibility** | Whether a person/asset may perform a scoped activity based on training, documents, signatures, inspections |
| **Compliance Record** | An auditable instance of required activity (safety form, inspection, acknowledgement, etc.) |
| **Evidence** | Durable proof (signature package, document version, completion record) suitable for audit |
| **Corrective Action** | Tracked remediation with owner, due date, and closure |
| **COR** | Certificate of Recognition (and regional equivalents)—audit framework mapped to operational evidence |
| **Workflow** | Durable business process enforcing assign → complete → review → close → escalate |
| **Module** | Deployable logical package owning one bounded context inside the monolith |

---

## 3. Bounded Context Map

### 3.1 Context Catalog

| # | Bounded Context | Module Name | Strategic Type |
| --- | --- | --- | --- |
| 1 | **Tenancy & Organization** | `tenancy` | Generic |
| 2 | **Identity & Access** | `identity` | Generic |
| 3 | **Projects** | `projects` | Supporting |
| 4 | **Workforce (People)** | `workforce` | Supporting |
| 5 | **Safety Operations** | `safety` | Core |
| 6 | **Equipment Compliance** | `equipment` | Core |
| 7 | **Document Control** | `documents` | Supporting |
| 8 | **Digital Evidence (Signatures)** | `signatures` | Core |
| 9 | **Training & Competency** | `training` | Core |
| 10 | **COR Audit Readiness** | `cor_audit` | Core |
| 11 | **Notifications** | `notifications` | Supporting |
| 12 | **Workflow Orchestration** | `workflows` | Generic |
| 13 | **Analytics & Insights** | `analytics` | Generic |
| 14 | **Platform Audit** | `audit` | Generic |

### 3.2 Context Map (Relationships)

```text
[Tenancy & Organization] —— shared kernel/ids ——> most contexts (TenantId, CompanyId)
[Identity & Access] —— enforces ——> all command surfaces
[Projects] <—— customer/supplier ——> Workforce, Safety, Equipment, Documents, Training, COR
[Workforce] <—— customer/supplier ——> Safety, Training, Projects, Equipment (operators)
[Document Control] <—— published language ——> Safety, Training, Equipment, COR, Signatures
[Digital Evidence] <—— published language ——> Safety, Documents, Training, Equipment
[Training & Competency] —— publishes eligibility facts ——> Projects, Safety, Workforce consumers
[Equipment Compliance] —— publishes readiness facts ——> Projects, Safety
[Safety Operations] —— publishes evidence/events ——> COR, Analytics, Notifications
[COR Audit] —— conformist to evidence publishers ——> Safety, Training, Documents, Equipment, Signatures
[Workflow Orchestration] —— process manager / orchestrator ——> core contexts via activities
[Notifications] —— conformist ——> events from all operational contexts
[Analytics] —— separate way / ACL ——> projected from integration events
[Platform Audit] —— open host ——> append-only audit from all modules
```

### 3.3 Relationship Patterns Used

| Pattern | Where applied |
| --- | --- |
| **Published Language** | Integration events + anti-corruption-friendly IDs and evidence references |
| **Customer/Supplier** | Projects supplies scope; Safety/Training/Equipment are customers of project membership |
| **Conformist** | Notifications and COR consume upstream event schemas carefully versioned |
| **Anti-Corruption Layer** | Analytics and future external ERP/LMS integrations |
| **Process Manager / Orchestration** | Workflow Orchestration (Temporal) coordinates multi-module processes |
| **Shared Kernel (minimal)** | Only stable identifiers and cross-cutting primitives (`TenantId`, `ProjectId`, `PersonId`, `Instant`, `Money` if needed)—never shared mutable models |

### 3.4 Hard Rules for Independence

1. **No cross-module table joins** in application code. Read models may denormalize via events.
2. **No shared mutable domain models** across modules.
3. **Foreign references are IDs + optional snapshots**, not object graphs.
4. **Each module owns its schema** (schema-per-module or schema prefix with ownership enforcement).
5. **Commands are accepted only by the owning module.**
6. **Workflows call public activities/interfaces**, never private functions.
7. **Events are versioned**; consumers must tolerate additive evolution.
8. **Redis is never a domain store**; PostgreSQL (or module-owned durable store) is system of record.
9. **Business rules live in the owning domain module**, not in React, Go workers, or workflow glue beyond orchestration.
10. **Audit logging is mandatory** for compliance-significant commands.

---

## 4. Platform Interaction Model

### 4.1 Synchronous Public Interfaces

Used when a module needs an **immediate, consistent answer** from another module (authorization checks, existence, eligibility query, document version resolve).

Rules:

- Interface is defined by the **owning** module.
- Callers depend on the interface, not the implementation.
- Prefer coarse queries that return decision-oriented DTOs (`EligibilityDecision`, `DocumentVersionRef`), not internal entities.

### 4.2 Domain & Integration Events (NATS)

Used when other modules should **react** without participating in the same transaction.

Rules:

- Publish **after** successful persistence in the owning module.
- Events name **business facts** in past tense (`SafetyActivityCompleted`).
- Include tenant, actor, correlation/causation IDs, and stable resource IDs.
- Do not include another module’s internal aggregates.

### 4.3 Temporal Workflows

Used for **durable multi-step business processes**: assignments, reminders, escalations, multi-party signatures, audit package generation, corrective action SLAs.

Rules:

- Workflows **orchestrate**; domains **decide**.
- Activities invoke module public commands/queries.
- Timers, retries, and compensation live in workflows.
- Never bypass Temporal for multi-step compliance processes that require durability.

### 4.4 Typical Interaction Sequence

```text
Actor → API (Identity enforces access)
     → Owning module command
     → Aggregate invariants
     → Persist + outbox
     → Domain events
     → (optional) Temporal signal/start
     → Downstream: Notifications, Analytics, COR projections, eligibility updates
     → Platform Audit append
```

---

## 5. Bounded Context Specifications

For each context: purpose, ownership, ubiquitous language, aggregates, entities, value objects, domain events, and interactions.

---

### 5.1 Tenancy & Organization

**Purpose:** Define the customer boundary, companies, org structure, and module enablement.

**Ownership:** `tenancy` module team / platform foundation.

**Ubiquitous language:** Tenant, Company, OrgUnit, Partnership, ModuleEntitlement.

#### Aggregate Roots

| Aggregate | Responsibility |
| --- | --- |
| **Tenant** | Lifecycle of a customer workspace; status; region defaults; entitlements |
| **Company** | Legal/operating company profile linked to tenant (owner or partner companies) |
| **OrgUnit** | Hierarchical business unit structure within a tenant |

#### Entities

- `TenantMembership` (company affiliation to tenant, if modeled inside Tenant)
- `OrgUnitAssignment`
- `ModuleEntitlement`

#### Value Objects

- `TenantId`, `CompanyId`, `OrgUnitId`
- `TenantStatus` (Active, Suspended, Closed)
- `RegionCode` (CA, US, AU, NZ, …)
- `CompanyType` (Prime, Subcontractor, Crane, Forming, Civil, Industrial, Other)
- `DisplayName`, `LegalName`
- `Address` (optional structured)

#### Domain Events

- `TenantProvisioned`
- `TenantSuspended`
- `TenantReactivated`
- `CompanyRegistered`
- `CompanyUpdated`
- `OrgUnitCreated`
- `OrgUnitMoved`
- `ModuleEntitlementChanged`

#### Interactions

| Direction | Mechanism | Counterpart |
| --- | --- | --- |
| Out | Events | Identity (bootstrap roles), Analytics, Audit |
| Out | Public query | All modules needing tenant/company validation |
| In | Commands | Administration UI / onboarding workflows |

---

### 5.2 Identity & Access

**Purpose:** Authenticate principals and authorize actions with least-privilege, project-scoped RBAC.

**Ownership:** `identity` module team / security.

**Ubiquitous language:** Principal, Role, Permission, Scope, Session, SSO Link.

#### Aggregate Roots

| Aggregate | Responsibility |
| --- | --- |
| **Principal** | Login identity bound to a person reference and tenant |
| **RoleDefinition** | Named role with permission set (tenant-customizable within policy) |
| **AccessGrant** | Binding of principal ↔ role ↔ scope (tenant / org / project) |
| **Session** (if domain-managed) | Session lifecycle and revocation |

#### Entities

- `PermissionBinding`
- `ScopeBinding`
- `ExternalIdentityLink` (SSO subject)

#### Value Objects

- `PrincipalId`, `RoleId`, `PermissionCode`
- `AccessScope` (TenantScope, OrgUnitScope, ProjectScope)
- `AuthProviderRef`
- `SessionId`
- `IpAddress`, `UserAgent` (for audit metadata, not authorization logic)

#### Domain Events

- `PrincipalRegistered`
- `PrincipalDeactivated`
- `RoleDefinitionChanged`
- `AccessGranted`
- `AccessRevoked`
- `SessionRevoked`
- `SsoLinkEstablished`

#### Interactions

| Direction | Mechanism | Counterpart |
| --- | --- | --- |
| In | Public authz queries | All modules (permission checks at application edge) |
| Out | Events | Audit, Notifications (security alerts) |
| In | Person reference | Workforce (`PersonId`) — ID only |

**Note:** Identity does not own HR attributes. It owns credentials, roles, and grants.

---

### 5.3 Projects

**Purpose:** Scope compliance work to construction undertakings; manage participation and required controls.

**Ownership:** `projects` module team.

**Ubiquitous language:** Project, Site/Area, Participant, RequiredControl, ProjectStatus.

#### Aggregate Roots

| Aggregate | Responsibility |
| --- | --- |
| **Project** | Project lifecycle, location/region, status, membership, required controls |
| **ProjectTemplate** | Reusable setup pattern for controls and defaults |

#### Entities

- `ProjectParticipant` (company participation: prime/sub/etc.)
- `ProjectMembership` (person assignment with project role)
- `RequiredControl` (required safety activity types, training requirements refs, document acknowledgement refs, equipment rules refs—as IDs)
- `ProjectArea` (optional sub-sites)

#### Value Objects

- `ProjectId`, `ProjectCode`
- `ProjectStatus` (Planning, Active, OnHold, Closed, Archived)
- `ParticipationRole` (Prime, Subcontractor, Supplier, Other)
- `ProjectRole` (PM, Supervisor, Safety, Worker, Viewer, …)
- `GeoLocation` / `SiteAddress`
- `ControlRef` (typed reference to another module’s requirement ID)
- `DateRange`

#### Domain Events

- `ProjectCreated`
- `ProjectActivated`
- `ProjectPutOnHold`
- `ProjectClosed`
- `ProjectArchived`
- `ParticipantAdded`
- `ParticipantRemoved`
- `MembershipGranted`
- `MembershipRevoked`
- `RequiredControlDefined`
- `RequiredControlRemoved`
- `ProjectTemplatePublished`

#### Interactions

| Direction | Mechanism | Counterpart |
| --- | --- | --- |
| Out | Events | Safety, Training, Equipment, Documents, COR, Analytics, Notifications |
| In | Queries | Eligibility consumers ask Workforce/Training/Equipment via their interfaces; Projects stores requirement *refs* only |
| In | Workflow | Project onboarding workflow |

Projects **do not** embed training completion or inspection logic; they declare requirements and consume eligibility decisions.

---

### 5.4 Workforce (People)

**Purpose:** System of record for people profiles, employment/contractor relationships, and crew structures.

**Ownership:** `workforce` module team.

**Ubiquitous language:** Person, Employment, ContractorEngagement, Crew, Trade, WorkerProfile.

#### Aggregate Roots

| Aggregate | Responsibility |
| --- | --- |
| **Person** | Identity profile of a human in the compliance OS (not login secrets) |
| **Crew** | Supervisor-oriented grouping of people for operational work |

#### Entities

- `Employment` (internal employee relationship to a company)
- `ContractorEngagement` (person engaged via contracting company)
- `TradeAssignment`
- `CrewMembership`
- `EmergencyContact` (if retained as entity under Person)

#### Value Objects

- `PersonId`
- `PersonName`
- `ContactInfo` (email, phone)
- `TradeCode`
- `EmploymentStatus`
- `EngagementPeriod`
- `WorkerClassification` (Employee, Contractor, Visitor, Temporary)

#### Domain Events

- `PersonRegistered`
- `PersonUpdated`
- `PersonDeactivated`
- `EmploymentStarted`
- `EmploymentEnded`
- `ContractorEngagementStarted`
- `ContractorEngagementEnded`
- `CrewCreated`
- `CrewMembershipChanged`

#### Interactions

| Direction | Mechanism | Counterpart |
| --- | --- | --- |
| Out | Events | Identity (provisioning hints), Projects (membership validation), Training, Safety, Analytics |
| In | Public query | Modules resolving names/trades for display snapshots |
| Out | Public query | “Does this PersonId exist and belong to Company X?” |

Workforce **does not** own training completions or safety records; it owns who people are and how they relate to companies/crews.

---

### 5.5 Safety Operations

**Purpose:** Plan, execute, review, and close safety compliance activities; manage corrective actions and escalations.

**Ownership:** `safety` module team (core).

**Ubiquitous language:** SafetyActivity, ActivityType, Hazard, Attendance, CorrectiveAction, Severity, Incident.

#### Aggregate Roots

| Aggregate | Responsibility |
| --- | --- |
| **SafetyActivity** | A single compliance activity instance (FLHA/JSA, toolbox talk, observation, inspection, incident report, etc.) |
| **ActivityTypeDefinition** | Configurable type schema/workflow binding for activities within tenant policy |
| **CorrectiveAction** | Remediation lifecycle with ownership and due dates |
| **IncidentCase** (P1+) | Investigation wrapper linking activities, evidence, and actions |

#### Entities

- `ActivityParticipant` / `AttendanceEntry`
- `HazardEntry`
- `ActivitySection` / `ResponseEntry` (structured answers)
- `ActivityAttachmentRef` (document/object refs)
- `InvestigationStep` (under IncidentCase)
- `ActionUpdate` (progress notes under CorrectiveAction)

#### Value Objects

- `SafetyActivityId`, `ActivityTypeId`, `CorrectiveActionId`, `IncidentCaseId`
- `ActivityStatus` (Draft, InProgress, Submitted, UnderReview, Closed, Voided)
- `SeverityLevel`
- `HazardCategory`
- `DueDate`
- `ClosureReason`
- `OfflineOrigin` (client mutation id / device metadata refs)
- `SignatureRequestRef` (reference to signatures module)
- `ProjectRef`, `PersonRef`, `EquipmentRef` (IDs + optional display snapshot)

#### Domain Events

- `SafetyActivityOpened`
- `SafetyActivityUpdated`
- `SafetyActivitySubmitted`
- `SafetyActivityReviewed`
- `SafetyActivityClosed`
- `SafetyActivityVoided`
- `AttendanceRecorded`
- `CorrectiveActionOpened`
- `CorrectiveActionAssigned`
- `CorrectiveActionCompleted`
- `CorrectiveActionOverdue` (may be workflow-emitted fact confirmed by domain)
- `CorrectiveActionClosed`
- `IncidentCaseOpened`
- `IncidentCaseClosed`
- `CriticalRiskRaised`

#### Interactions

| Direction | Mechanism | Counterpart |
| --- | --- | --- |
| In | Queries | Projects (scope), Workforce (participants), Equipment (asset refs), Documents (controlled docs), Training (eligibility) |
| Out | Commands via interface | Signatures (create signature package), Documents (attach) |
| Out | Events | COR, Analytics, Notifications, Audit |
| In/Out | Temporal | Activity SLA, review escalation, multi-party acknowledgement workflows |

---

### 5.6 Equipment Compliance

**Purpose:** Govern equipment identity, inspections, certifications, readiness, and custody signals.

**Ownership:** `equipment` module team (core).

**Ubiquitous language:** Asset, Inspection, Certification, Readiness, OutOfService, Custody.

#### Aggregate Roots

| Aggregate | Responsibility |
| --- | --- |
| **Asset** | Equipment identity, type, ownership, status, project assignment |
| **Inspection** | Inspection/pre-use check instance against a checklist |
| **CertificationRecord** | Certificate/document-backed compliance credential with expiry |

#### Entities

- `ChecklistItemResult`
- `AssetAssignment` (to project/person)
- `CustodyTransfer`
- `DefectFinding`

#### Value Objects

- `AssetId`, `AssetTag`, `SerialNumber`
- `AssetType`
- `AssetStatus` (Available, Assigned, OutOfService, Retired)
- `ReadinessState` (Ready, Restricted, Blocked)
- `InspectionStatus`
- `ExpiryDate`
- `ChecklistDefinitionRef`
- `DocumentVersionRef`
- `FailureCode`

#### Domain Events

- `AssetRegistered`
- `AssetUpdated`
- `AssetAssignedToProject`
- `AssetUnassigned`
- `AssetTakenOutOfService`
- `AssetReturnedToService`
- `AssetRetired`
- `InspectionStarted`
- `InspectionPassed`
- `InspectionFailed`
- `CertificationRecorded`
- `CertificationExpiring`
- `CertificationExpired`
- `CustodyTransferred`

#### Interactions

| Direction | Mechanism | Counterpart |
| --- | --- | --- |
| In | Queries | Projects, Workforce (operator), Documents |
| Out | Events | Safety (usable-asset signals), Projects, Notifications, COR, Analytics |
| Out | Public query | `GetAssetReadiness(AssetId, ProjectId)` |
| In | Temporal | Expiry watchers, inspection due workflows |

---

### 5.7 Document Control

**Purpose:** Controlled documents with versioning, effective dating, acknowledgements, and secure object references.

**Ownership:** `documents` module team.

**Ubiquitous language:** Document, DocumentVersion, ControlledCopy, Acknowledgement, Distribution, RetentionClass.

#### Aggregate Roots

| Aggregate | Responsibility |
| --- | --- |
| **Document** | Logical document identity and access policy |
| **DocumentVersion** | Immutable version content reference + effective window |
| **AcknowledgementRequest** | Requirement for people/projects to acknowledge a specific version |
| **DistributionList** (P1) | Controlled distribution tracking |

#### Entities

- `Acknowledgement` (person acknowledgement instance)
- `DocumentCollectionMembership`
- `AccessRule`

#### Value Objects

- `DocumentId`, `DocumentVersionId`
- `DocumentCategory` (Policy, SWP, SDS, Permit, SiteDoc, Certificate, Other)
- `VersionNumber`
- `EffectivePeriod`
- `ObjectStorageRef` (R2 key / checksum / content type)—reference only
- `Checksum`
- `RetentionClass`
- `AcknowledgementStatus`

#### Domain Events

- `DocumentCreated`
- `DocumentVersionPublished`
- `DocumentVersionSuperseded`
- `DocumentRetired`
- `AcknowledgementRequested`
- `DocumentAcknowledged`
- `DistributionIssued`
- `RetentionHoldApplied`

#### Interactions

| Direction | Mechanism | Counterpart |
| --- | --- | --- |
| Out | Events | Training, Safety, Equipment, COR, Notifications, Analytics |
| In | Storage | Object storage via platform adapter; domain stores refs only |
| Out | Public query | `GetCurrentEffectiveVersion(DocumentId, atTime)` |
| In | Signatures | Acknowledgement may require signature package |

---

### 5.8 Digital Evidence (Signatures)

**Purpose:** Produce audit-grade signature evidence bound to identity, time, document/record version, and context.

**Ownership:** `signatures` module team (core).

**Ubiquitous language:** SignaturePackage, Signer, SignatureEvidence, SigningPolicy, SigningOrder.

#### Aggregate Roots

| Aggregate | Responsibility |
| --- | --- |
| **SignaturePackage** | Multi-signer request bound to a subject record/document version |
| **SigningPolicy** | Tenant rules for required assurance by process type |

#### Entities

- `SignerSlot`
- `CapturedSignature`
- `EvidenceArtifact` (immutable evidence snapshot metadata)

#### Value Objects

- `SignaturePackageId`
- `SignerId` / `PersonRef`
- `SigningStatus` (Pending, PartiallySigned, Completed, Voided, Expired)
- `SigningOrder` (Parallel, Sequential)
- `SubjectRef` (typed reference: SafetyActivity, DocumentVersion, Inspection, TrainingCompletion, …)
- `SignatureBlobRef` (storage reference)
- `SignedAt`
- `AssuranceLevel`
- `DeviceSessionMeta` (limited, policy-driven)

#### Domain Events

- `SignaturePackageCreated`
- `SignerNotified` (optional; may be notification-only)
- `SignatureCaptured`
- `SignaturePackageCompleted`
- `SignaturePackageVoided`
- `SignaturePackageExpired`
- `SigningPolicyChanged`

#### Interactions

| Direction | Mechanism | Counterpart |
| --- | --- | --- |
| In | Commands from | Safety, Documents, Training, Equipment |
| Out | Events | Originating modules, COR, Audit, Analytics |
| In | Temporal | Multi-signer sequential workflows, expiry |
| In | Identity | Assurance of signer principal |

Signatures never own the business meaning of the signed subject; they own **proof of assent**.

---

### 5.9 Training & Competency

**Purpose:** Define requirements, track completions/expiries, and publish eligibility-related competency facts.

**Ownership:** `training` module team (core).

**Ubiquitous language:** Requirement, Completion, Competency, Expiry, Evidence, Assignment Rule.

#### Aggregate Roots

| Aggregate | Responsibility |
| --- | --- |
| **TrainingCourse** | Catalog item representing a training/competency unit |
| **TrainingRequirement** | Rule assigning a course to role/trade/project/person |
| **TrainingCompletion** | Evidence that a person completed a course, with validity window |

#### Entities

- `RequirementScope` (embedded rules)
- `CompletionEvidenceAttachment`
- `Waiver` (if allowed; tightly controlled)

#### Value Objects

- `CourseId`, `RequirementId`, `CompletionId`
- `ValidityPeriod`
- `CompetencyStatus` (Valid, Expiring, Expired, Missing)
- `AssignmentDimension` (Role, Trade, Project, Person)
- `EvidenceRef`
- `EligibilityContribution` (value published to consumers)

#### Domain Events

- `TrainingCourseDefined`
- `TrainingRequirementAssigned`
- `TrainingRequirementRemoved`
- `TrainingCompletionRecorded`
- `TrainingCompletionExpiring`
- `TrainingCompletionExpired`
- `TrainingCompletionRevoked`
- `CompetencyGapDetected` (projection-friendly fact)

#### Interactions

| Direction | Mechanism | Counterpart |
| --- | --- | --- |
| Out | Events / queries | Projects, Safety, Workforce consumers, COR, Notifications |
| In | Documents / Signatures | Certificate uploads and acknowledgements |
| In | Temporal | Expiry watch, re-assignment, reminder cadence |
| Out | Public query | `GetPersonCompetency(PersonId, ProjectId)` |

---

### 5.10 COR Audit Readiness

**Purpose:** Map operational evidence to COR (and regional equivalent) elements; measure readiness; produce evidence packages.

**Ownership:** `cor_audit` module team (core).

**Ubiquitous language:** Framework, AuditElement, EvidenceMapping, ReadinessScore, Gap, EvidencePackage, InternalAudit.

#### Aggregate Roots

| Aggregate | Responsibility |
| --- | --- |
| **AuditFramework** | Versioned COR/equivalent program definition |
| **ReadinessProfile** | Tenant/project readiness against a framework |
| **EvidencePackage** | Exportable bundle with provenance for audit submission |
| **InternalAudit** (P1) | Internal audit cycle, findings, remediation links |

#### Entities

- `AuditElement`
- `EvidenceMapping` (element ↔ evidence refs from other modules)
- `GapItem`
- `PackageItem`
- `InternalFinding`

#### Value Objects

- `FrameworkId`, `FrameworkVersion`
- `ElementCode`
- `CoverageStatus` (Covered, Partial, Missing, NotApplicable)
- `ReadinessScore`
- `ProvenanceRef` (module, aggregate id, event id, hash)
- `PackageStatus`

#### Domain Events

- `AuditFrameworkPublished`
- `EvidenceLinkedToElement`
- `EvidenceUnlinked`
- `ReadinessRecalculated`
- `GapOpened`
- `GapClosed`
- `EvidencePackageRequested`
- `EvidencePackageGenerated`
- `InternalAuditOpened`
- `InternalFindingRecorded`
- `InternalAuditClosed`

#### Interactions

| Direction | Mechanism | Counterpart |
| --- | --- | --- |
| In | Conformist events | Safety, Training, Documents, Equipment, Signatures, Projects |
| Out | Temporal | Package generation workflow (long-running assembly) |
| Out | Events | Notifications, Analytics |
| ACL | Mapping layer | Translates upstream evidence types into element coverage without importing foreign aggregates |

COR is a **consumer and organizer of proof**, not a second system of record for safety/training data.

---

### 5.11 Notifications

**Purpose:** Deliver timely, preference-aware notifications for assignments, expiries, escalations, and approvals.

**Ownership:** `notifications` module team.

**Ubiquitous language:** Notification, Channel, Preference, Delivery, Digest.

#### Aggregate Roots

| Aggregate | Responsibility |
| --- | --- |
| **Notification** | A single notification to a recipient about a business fact |
| **NotificationPreference** | User/tenant channel and quiet-hour preferences within policy |
| **DeliveryRule** (tenant) | What events are notifiable at what priority |

#### Entities

- `DeliveryAttempt`
- `DigestBucket`

#### Value Objects

- `NotificationId`
- `Channel` (InApp, Email, Push, SMS)
- `Priority`
- `RecipientRef`
- `TemplateCode`
- `DedupKey`
- `QuietHours`

#### Domain Events

- `NotificationCreated`
- `NotificationDispatched`
- `NotificationDelivered`
- `NotificationFailed`
- `NotificationRead`
- `PreferenceUpdated`

#### Interactions

| Direction | Mechanism | Counterpart |
| --- | --- | --- |
| In | Events | All operational modules |
| Out | Workers (Go) | Email/push providers — **no business rules** in workers beyond delivery |
| In | Identity | Recipient routing |

---

### 5.12 Workflow Orchestration

**Purpose:** Durable orchestration of multi-step compliance processes; timers; escalations; saga-style coordination.

**Ownership:** `workflows` module / platform team.

**Ubiquitous language:** WorkflowDefinition, WorkflowInstance, ActivityCall, EscalationPolicy, Correlation.

**Important:** This context **orchestrates**; it does not own Safety/Training/Equipment invariants.

#### Aggregate Roots

| Aggregate | Responsibility |
| --- | --- |
| **WorkflowDefinition** | Tenant-visible template metadata bound to Temporal workflow types |
| **WorkflowInstance** | Platform tracking record for a running/completed business process (correlation, status, subject refs) |

#### Entities

- `WorkflowMilestone` (optional visibility projections)
- `EscalationStep`

#### Value Objects

- `WorkflowDefinitionId`, `WorkflowInstanceId`
- `TemporalRunRef`
- `WorkflowStatus`
- `SubjectRef`
- `EscalationPolicy`
- `CorrelationId`

#### Domain Events

- `WorkflowStarted`
- `WorkflowSignaled`
- `WorkflowCompleted`
- `WorkflowFailed`
- `WorkflowCancelled`
- `EscalationTriggered`

#### Interactions

| Direction | Mechanism | Counterpart |
| --- | --- | --- |
| Out | Activity calls | Public commands/queries on Safety, Signatures, Training, Equipment, Documents, COR, Notifications |
| In | Signals/commands | API and domain modules requesting process start |
| Constraint | Must not | Duplicate domain validation already enforced by aggregates |

---

### 5.13 Analytics & Insights

**Purpose:** Provide read-optimized insights and scorecards without burdening transactional modules.

**Ownership:** `analytics` module team.

**Ubiquitous language:** Metric, Scorecard, Projection, TimeBucket, Dimension, ReportDefinition.

#### Aggregate Roots

| Aggregate | Responsibility |
| --- | --- |
| **ReportDefinition** | Saved report/scorecard configuration |
| **AnalyticsSubscription** (optional) | Scheduled delivery of reports |

> Most analytical data is **not** modeled as write-side aggregates. It is projected into PostgreSQL read models and/or ClickHouse.

#### Entities

- `ReportSchedule`
- `DashboardWidget` (configuration only)

#### Value Objects

- `MetricKey`
- `DimensionKey` (Tenant, Project, Company, Trade, Region)
- `TimeBucket`
- `Score`
- `FilterSpec`

#### Domain Events

- `ReportDefinitionPublished`
- `AnalyticsProjectionRebuilt`
- `ScheduledReportDispatched`

#### Interactions

| Direction | Mechanism | Counterpart |
| --- | --- | --- |
| In | ACL + events | All operational modules |
| Out | Queries | Executive/PM dashboards |
| Constraint | Never | Writes back business state into core domains (insights may trigger workflows only via explicit commands) |

---

### 5.14 Platform Audit

**Purpose:** Immutable, append-oriented record of security and compliance-significant actions across the OS.

**Ownership:** `audit` module / platform security.

**Ubiquitous language:** AuditEntry, Actor, Action, Resource, Correlation, Integrity.

#### Aggregate Roots

| Aggregate | Responsibility |
| --- | --- |
| **AuditStream** (logical) | Tenant-scoped append stream; entries are immutable |

In practice, **AuditEntry** records are append-only entities/records under a stream partition; treat the stream as the consistency boundary for writes (append only).

#### Entities

- `AuditEntry`

#### Value Objects

- `AuditEntryId`
- `ActorRef` (principal/person/system)
- `ActionCode`
- `ResourceRef`
- `BeforeAfterHash` / `PayloadDigest`
- `CorrelationId`, `CausationId`
- `OccurredAt`

#### Domain Events

- `AuditEntryAppended` (optional fan-out; avoid noisy cycles)
- `AuditExportGenerated`

#### Interactions

| Direction | Mechanism | Counterpart |
| --- | --- | --- |
| In | Open host service | All modules must append on significant commands |
| Out | Exports | Admin, COR package provenance support |
| Constraint | Never | Mutate historical entries |

---

## 6. Aggregate Catalog (Quick Reference)

| Context | Aggregate Roots |
| --- | --- |
| Tenancy & Organization | Tenant, Company, OrgUnit |
| Identity & Access | Principal, RoleDefinition, AccessGrant, Session |
| Projects | Project, ProjectTemplate |
| Workforce | Person, Crew |
| Safety Operations | SafetyActivity, ActivityTypeDefinition, CorrectiveAction, IncidentCase |
| Equipment Compliance | Asset, Inspection, CertificationRecord |
| Document Control | Document, DocumentVersion, AcknowledgementRequest, DistributionList |
| Digital Evidence | SignaturePackage, SigningPolicy |
| Training & Competency | TrainingCourse, TrainingRequirement, TrainingCompletion |
| COR Audit Readiness | AuditFramework, ReadinessProfile, EvidencePackage, InternalAudit |
| Notifications | Notification, NotificationPreference, DeliveryRule |
| Workflow Orchestration | WorkflowDefinition, WorkflowInstance |
| Analytics & Insights | ReportDefinition, AnalyticsSubscription |
| Platform Audit | AuditStream (AuditEntry append model) |

---

## 7. Cross-Cutting Domain Concepts

### 7.1 Eligibility (Distributed Decision)

Eligibility is **not** a single aggregate owned by one module.

It is a **composed decision**:

```text
EligibilityDecision =
  ProjectMembership (Projects)
  + PersonActive (Workforce)
  + Competency (Training)
  + RequiredDocumentAcknowledgements (Documents)
  + AssetReadiness (Equipment) [when task involves equipment]
  + AccessGrant (Identity)
```

Composition occurs via:

- Synchronous queries to owning modules, or
- A workflow/application policy in the consuming use case, or
- A carefully owned **read model** updated by events (still not a second write model for source facts)

### 7.2 Evidence Provenance

Every evidence reference used by COR or exports should resolve to:

- Owning module
- Aggregate ID
- Version/hash when applicable
- Event ID / audit entry ID
- Timestamp

### 7.3 Offline Field Mutations

Offline support is an **application concern** with domain constraints:

- Aggregates define what may be created/updated offline.
- Client mutation IDs ensure idempotency.
- Conflict rules are domain-owned (e.g., void vs overwrite policies).
- Sync completes only through owning module commands.

### 7.4 Multi-Party Construction Reality

GC/Sub relationships are modeled as **Project participants** + **Access scopes**, not as duplicated people databases per contractor tool.

Each company remains a first-class `Company`; project participation grants scoped visibility.

---

## 8. Modular Monolith Packaging

### 8.1 Module Layout Principles

Each bounded context maps to one module with internal layers:

```text
module/
  domain/           # aggregates, VOs, domain events, domain services
  application/      # commands, queries, public interfaces
  infrastructure/   # persistence, bus, temporal activities adapters
  api/              # HTTP routes for this module (composition at host)
```

Host application wires modules; modules do not import each other’s `domain` or `infrastructure`.

### 8.2 Allowed Dependencies

```text
api/host → application interfaces of modules
module.application → module.domain
module.infrastructure → module.application/domain
module.application → other.module.application.public (interfaces only)
workflows → other.module.application.public
Go workers → delivery/jobs APIs only (no domain rule ownership)
Next.js → public HTTP APIs only (no business rules)
```

### 8.3 Data Ownership

| Rule | Practice |
| --- | --- |
| Schema ownership | One schema (or prefix) per module |
| Transactions | Single-module transactions by default |
| Multi-module consistency | Outbox events + workflows; avoid distributed DB transactions |
| Read models | Owned by consumer module; rebuildable from events |
| Files | R2 object refs owned by Documents/Signatures/etc.; binaries not in Postgres |

### 8.4 Evolution Independence Checklist

A module may evolve independently when:

1. Public interfaces remain backward compatible or are versioned.
2. Events are additive and versioned.
3. Foreign modules depend only on IDs and published contracts.
4. Migrations do not lock or rewrite other modules’ tables.
5. Feature flags/entitlements can disable the module without compiling it out of existence overnight (host may no-op routes).

Extracting a module to a separate service later is possible **only because** these rules were honored inside the monolith.

---

## 9. Core Process Blueprints (Cross-Context)

### 9.1 Daily Crew Safety Activity

```text
Projects (membership) → Workforce (crew)
  → Training (eligibility query)
  → Safety (open activity)
  → Signatures (acknowledgements)
  → Documents (controlled refs if required)
  → Workflows (review/escalation timers)
  → Notifications
  → COR (evidence mapping projection)
  → Audit
  → Analytics
```

### 9.2 Equipment Pre-Use

```text
Equipment (start inspection)
  → Documents (checklist/cert refs)
  → Signatures (operator sign-off)
  → Equipment (pass/fail → readiness)
  → Notifications (on fail/expiry)
  → Safety/Projects consumers of readiness
  → COR / Analytics / Audit
```

### 9.3 Training Expiry Impact

```text
Training (expiry workflow fires)
  → TrainingCompletionExpired event
  → Notifications
  → Projects/Safety eligibility read models update
  → COR gap projection may open
  → Audit
```

### 9.4 COR Evidence Package

```text
COR (request package)
  → Workflow orchestrates gather
  → Queries/evidence refs from Safety, Training, Documents, Equipment, Signatures
  → Package sealed with provenance
  → Notifications
  → Audit
```

---

## 10. Domain Event Taxonomy

### 10.1 Naming Convention

```text
<Context><Entity><PastTenseVerb>
```

Examples: `SafetyActivitySubmitted`, `CertificationExpired`, `SignaturePackageCompleted`.

### 10.2 Envelope (Logical)

Every event carries:

- `event_id`
- `event_type` + `event_version`
- `occurred_at`
- `tenant_id`
- `actor` (principal/person/system)
- `correlation_id` / `causation_id`
- `resource` (type + id)
- `payload` (context-specific, no foreign internals)

### 10.3 Event Categories

| Category | Examples | Typical consumers |
| --- | --- | --- |
| Lifecycle | Created/Activated/Closed | Analytics, COR, Notifications |
| Compliance fact | Submitted/Signed/Expired/Failed | COR, Eligibility projections |
| Assignment | Assigned/Revoked | Notifications, Workflows |
| Security | AccessGranted/SessionRevoked | Audit, Notifications |
| Orchestration | WorkflowStarted/EscalationTriggered | Notifications, Admin visibility |

---

## 11. Value Object Catalog (Shared Kernel — Minimal)

Only these cross-module primitives are candidates for a **minimal shared kernel** library (identifiers + pure values). They must remain immutable and free of behavior that encodes another module’s rules.

| VO / ID | Used by |
| --- | --- |
| `TenantId` | All |
| `CompanyId` | Tenancy, Projects, Workforce, Equipment |
| `ProjectId` | Projects + most operational contexts |
| `PersonId` | Workforce + consumers |
| `PrincipalId` | Identity + Audit |
| `CorrelationId` | All events/workflows |
| `Instant` / `Date` | All |
| `RegionCode` | Tenancy, Projects, COR frameworks |

Everything else remains **context-local** even if names coincide (`Status` enums are not shared).

---

## 12. Ownership Matrix

| Bounded Context | Owning Module | Primary Actors Served | System of Record For |
| --- | --- | --- | --- |
| Tenancy & Organization | `tenancy` | Admins | Tenants, companies, org units, entitlements |
| Identity & Access | `identity` | Security, all users | Principals, roles, grants, sessions |
| Projects | `projects` | PM, Admin, Supervisor | Projects, participation, required controls |
| Workforce | `workforce` | Admin, Supervisor | People, employments, crews |
| Safety Operations | `safety` | Worker, Supervisor, Safety | Activities, corrective actions, incidents |
| Equipment Compliance | `equipment` | Equipment mgr, Operator, Supervisor | Assets, inspections, certifications |
| Document Control | `documents` | Safety, Admin, PM | Documents, versions, acknowledgements |
| Digital Evidence | `signatures` | All signing actors | Signature packages & evidence |
| Training & Competency | `training` | Training admin, Worker, Supervisor | Courses, requirements, completions |
| COR Audit Readiness | `cor_audit` | Safety lead, Executive, Auditor | Frameworks, readiness, packages |
| Notifications | `notifications` | All users | Notification records & preferences |
| Workflow Orchestration | `workflows` | Platform + process owners | Workflow definitions/instances (not domain docs) |
| Analytics & Insights | `analytics` | Exec, PM, Safety | Projections & report defs |
| Platform Audit | `audit` | Security, Compliance | Audit entries |

---

## 13. Anti-Patterns (Explicitly Forbidden)

1. **Shared database models** used by multiple modules as their write model.
2. **React-encoded business invariants** (UI may validate UX only).
3. **Go workers deciding compliance outcomes** (delivery/transform only).
4. **Workflows re-implementing aggregate rules**.
5. **COR storing duplicate mutable copies** of safety/training records as authority.
6. **Using Redis as source of truth** for eligibility, sessions of record, or evidence.
7. **Chatty cross-module entity navigation** (`project.crew.workers.trainings.completions…` object graphs).
8. **Bypassing audit** on signature, approval, closure, access-change, or export actions.
9. **Reaching into another module’s SQL schema** for convenience joins.
10. **Tight temporal coupling** requiring synchronous multi-module write transactions for normal flows.

---

## 14. Phased Domain Delivery Alignment

| Phase | Domains emphasized |
| --- | --- |
| Foundation | Tenancy, Identity, Projects, Workforce, Documents, Signatures, Workflows, Audit, Notifications |
| Compliance MVP | Safety, Equipment, Training, COR (initial), Analytics (foundational) |
| Enterprise scale | Identity SSO depth, richer GC/Sub workflows, regional COR variants, IncidentCase depth |
| Intelligence & ecosystem | Analytics warehouse depth, ACLs to external systems, search upgrade consumers |

Boundaries above are designed so phases add aggregates/events inside modules rather than rewriting the map.

---

## 15. Success Criteria for This Architecture

The domain design is successful when:

1. A new compliance feature has an obvious owning context.
2. Teams can ship inside one module without coordinating schema changes in others.
3. Cross-module features are expressible as interface calls + events + workflows.
4. Audit and COR can prove provenance without breaking encapsulation.
5. Any module could be extracted later with event/API contracts as the seam.
6. Field offline flows commit through owning aggregates without corrupting invariants.
7. The product remains one OS—modular internally, cohesive externally.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Domain Architecture | Initial complete DDD domain design for Proven modular monolith |

---

*End of Domain Architecture*
