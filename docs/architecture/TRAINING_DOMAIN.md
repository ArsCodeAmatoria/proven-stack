# Proven — Training Domain Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Training Domain Architecture (DDD) |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Principal Architecture |
| **Audience** | Engineering, Product, Design, Training / Competency Admins |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Domain Model](./DOMAIN_MODEL.md), [Core Domain](./CORE_DOMAIN.md), [People Domain](./PEOPLE_DOMAIN.md), [Projects Domain](./PROJECTS_DOMAIN.md), [Documents Domain](./DOCUMENTS_DOMAIN.md), [Safety Domain](./SAFETY_DOMAIN.md), [System Architecture](./SYSTEM_ARCHITECTURE.md), [PRD](../PRD.md) |

---

## 1. Purpose

This document defines the **Training** bounded context for Proven.

Training is a **strategic core domain** of the Construction Compliance Operating System. It is the system of record for courses, orientations, competencies, evaluations, certificates, assignments, expiry and renewals, and the training matrix—producing eligibility signals that Safety, Equipment, Projects, and People consume—while using Documents for learning materials/certificates binaries and Signatures for sealed attestations.

**Documentation only — no implementation.**

---

## 2. Bounded Context

### 2.1 Context Card

| Field | Value |
| --- | --- |
| **Name** | Training & Competency |
| **Module** | `training` |
| **Strategic type** | Core domain (differentiating) |
| **Product metaphor** | Competency = time-bounded proof a person may do scoped work |
| **System of record for** | Course catalog, orientations, competency definitions, evaluations, training completions/certificates records, expiry state, training requirements, assignments, renewals, training matrix projections, toolbox *training content* library, training reporting projections |
| **Not system of record for** | Person profiles (People), controlled document versioning (Documents), signature evidence packages (Signatures), Safety toolbox *talk instances* (Safety), AuthZ (Core), notification delivery (Notifications) |

### 2.2 Context Map

```text
Core (authz, membership, files, audit)
People (PersonId, trades, workforce roles)
Projects (required training controls)
Documents (materials & certificate PDFs)
Signatures (seal evaluations/acks)
        │
        ▼
┌────────────────────────────────────────────┐
│                 TRAINING                   │
│  Courses · Orientations · Competencies     │
│  Evaluations · Certs · Matrix · Renewals   │
└──────────────────┬─────────────────────────┘
                   │
     Safety / Equipment eligibility gates
     People competency projections
     COR · Analytics · Notifications · Workflows
```

### 2.3 Ubiquitous Language

| Term | Meaning |
| --- | --- |
| **Course** | Catalog learning/competency unit |
| **Orientation** | Course subtype for site/company onboarding |
| **Competency** | Named capability that may map to one or more courses/evaluations |
| **Evaluation** | Assessment of skill/knowledge (quiz, practical sign-off, observation) |
| **Certificate / Completion** | Record that a person satisfied a course/competency for a validity window |
| **Requirement** | Rule that a course/competency is mandatory for a scope |
| **Assignment** | Concrete obligation for a person to complete training |
| **Expiry** | End of validity for a completion |
| **Renewal** | Process to re-establish validity before/after expiry |
| **Training Matrix** | Cross-tab of people/roles/trades × required competencies/courses and status |
| **Toolbox Library** | Reusable training toolbox talk *content* packs (topics, talking points)—not the sealed crew talk event |
| **Eligibility Contribution** | Training’s input into composed eligibility decisions |

### 2.4 Toolbox Library vs Safety Toolbox Talks

| Concept | Owner |
| --- | --- |
| Toolbox topic content, packs, curricula | **Training** (`ToolboxLibraryItem`) |
| Crew toolbox talk instance, attendance, seals | **Safety** (`SafetyActivity`) |

Safety may reference a Training toolbox library item as the topic source when starting a talk.

---

## 3. Responsibilities (Mapped to Ownership)

| Responsibility | Training owns? | Clarification |
| --- | --- | --- |
| **Courses** | Yes | Catalog + metadata + validity rules |
| **Orientations** | Yes | Course/requirement subtype for onboarding |
| **Competencies** | Yes | Competency definitions + fulfillment rules |
| **Evaluations** | Yes | Evaluation definitions + results tied to persons |
| **Certificates** | Yes (records) | Binary may be Documents/Files; record + validity in Training |
| **Expiry Tracking** | Yes | Authoritative validity windows + workflows |
| **Training Matrix** | Yes | Authoritative matrix computation/projections |
| **Toolbox Library** | Yes | Content library for toolbox topics |
| **Digital Signatures** | Requests | Signatures owns evidence; Training subjects evaluations/completions |
| **Training Assignments** | Yes | Person-level obligations |
| **Renewals** | Yes | Renewal campaigns/workflows |
| **Reporting** | Yes (training reports) + events to Analytics | |

People may **display** matrix/cert cards; Training remains source of truth for status.

---

## 4. Aggregate Roots

| Aggregate | Responsibility |
| --- | --- |
| **TrainingCourse** | Course/orientation catalog item, duration, validity policy, materials refs, evaluation requirements |
| **CompetencyDefinition** | Named competency and how it is satisfied (courses/evaluations) |
| **EvaluationDefinition** | Quiz/practical/observation template |
| **EvaluationAttempt** | A person’s attempt/result for an evaluation |
| **TrainingRequirement** | Mandatory rule by role/trade/project/company/person |
| **TrainingAssignment** | Concrete assigned work for a person (from requirement or manual) |
| **TrainingCompletion** | Authoritative completion/certificate record with validity |
| **RenewalCase** | Renewal workflow instance for an expiring/expired completion |
| **ToolboxLibraryItem** | Reusable toolbox topic content pack |
| **TrainingMatrixProjection** | Matrix read model (tenant/project scoped) |
| **TrainingReportingProjection** | Report read models |

---

## 5. Entities

### 5.1 Under Course / Competency

| Entity | Parent | Description |
| --- | --- | --- |
| **CourseMaterialRef** | TrainingCourse | DocumentVersionId / FileObjectId |
| **CourseCompetencyLink** | TrainingCourse | Links course → competencies granted on completion |
| **ValidityPolicy** | TrainingCourse | Duration, grace, renewable flag |
| **FulfillmentRule** | CompetencyDefinition | All-of / any-of courses or evaluations |
| **OrientationProfile** | TrainingCourse | Site/company orientation flags, project-scoped defaults |

### 5.2 Under Evaluation

| Entity | Parent | Description |
| --- | --- | --- |
| **EvaluationSection** | EvaluationDefinition | Questions / practical criteria |
| **EvaluationScore** | EvaluationAttempt | Score/outcome |
| **EvaluatorRef** | EvaluationAttempt | Assessor PersonId |
| **SignaturePackageRef** | EvaluationAttempt | Practical sign-off seal |
| **EvidenceAttachmentRef** | EvaluationAttempt | Photos/files |

### 5.3 Under Requirements / Assignments / Completions

| Entity | Parent | Description |
| --- | --- | --- |
| **RequirementScope** | TrainingRequirement | Role, trade, project, company, person dimensions |
| **AssignmentSource** | TrainingAssignment | FromRequirement / Manual / Renewal |
| **CompletionEvidence** | TrainingCompletion | DocumentVersion, FileObject, external ref, evaluation attempt |
| **Waiver** | TrainingCompletion / Assignment | Rare, permissioned exception with expiry |
| **RenewalStep** | RenewalCase | Assigned course/eval to restore validity |

### 5.4 Under Toolbox Library & Matrix

| Entity | Description |
| --- | --- |
| **ToolboxTopicSection** | Talking points, hazards refs (text), resource links |
| **ToolboxTradeTag** | Trade targeting |
| **MatrixRow** | Person or role/trade dimension |
| **MatrixCell** | Required item × status (Valid/Expiring/Expired/Missing/Waived) |

---

## 6. Value Objects

- `CourseId`, `CompetencyId`, `EvaluationDefinitionId`, `EvaluationAttemptId`
- `RequirementId`, `AssignmentId`, `CompletionId`, `RenewalCaseId`
- `ToolboxLibraryItemId`
- `PersonId`, `ProjectId`, `CompanyId`, `DocumentVersionId`, `FileObjectId`, `SignaturePackageId`
- `CourseKind` — Course | Orientation | CertificationProgram | Other
- `CompetencyStatus` — Valid | Expiring | Expired | Missing | Waived
- `AssignmentStatus` — Assigned | InProgress | Completed | Cancelled | Overdue
- `EvaluationOutcome` — Pass | Fail | Incomplete
- `CompletionSource` — InternalEval | ExternalCert | Import | SupervisorAttestation
- `ValidityPeriod` — validFrom, validTo
- `ExpiryWindow` — expiringSoon threshold
- `RequirementDimension` — Role | Trade | Project | Company | Person
- `RenewalStatus` — Open | Completed | Cancelled | Overdue

---

## 7. Relationships

```text
CompetencyDefinition ◄──fulfilled by── TrainingCourse / EvaluationDefinition
TrainingRequirement ──scopes──► Person dimensions (role/trade/project/…)
        │ generates
        ▼
TrainingAssignment ──for──► Person
        │ completed via
        ▼
EvaluationAttempt and/or evidence upload
        │ results in
        ▼
TrainingCompletion ──validity──► CompetencyStatus
        │ expiring
        ▼
RenewalCase ──creates──► new Assignment

TrainingCourse ──materials──► Documents / Files
TrainingCompletion ──certificate binary──► Documents / Files
EvaluationAttempt ──seal──► Signatures

ToolboxLibraryItem ──referenced by──► Safety toolbox activities (topic source)

TrainingMatrixProjection ◄── events from completions/requirements/assignments
People.CompetencyProfileProjection ◄── Training events (display)
```

---

## 8. Domain Events

### 8.1 Catalog

- `TrainingCourseDefined` / `Updated` / `Retired`
- `CompetencyDefinitionDefined` / `Updated` / `Retired`
- `EvaluationDefinitionPublished`
- `ToolboxLibraryItemDefined` / `Updated` / `Retired`

### 8.2 Requirements & Assignments

- `TrainingRequirementAssigned`
- `TrainingRequirementRemoved`
- `TrainingAssignmentCreated`
- `TrainingAssignmentUpdated`
- `TrainingAssignmentCompleted`
- `TrainingAssignmentOverdue`
- `TrainingAssignmentCancelled`

### 8.3 Evaluations & Completions

- `EvaluationAttemptStarted`
- `EvaluationAttemptSubmitted`
- `EvaluationAttemptPassed`
- `EvaluationAttemptFailed`
- `TrainingCompletionRecorded`
- `TrainingCompletionExpiring`
- `TrainingCompletionExpired`
- `TrainingCompletionRevoked`
- `TrainingWaiverGranted`
- `TrainingWaiverExpired`

### 8.4 Renewals & Matrix

- `RenewalCaseOpened`
- `RenewalCaseCompleted`
- `RenewalCaseOverdue`
- `CompetencyGapDetected`
- `CompetencyGapResolved`
- `TrainingMatrixRebuilt`

---

## 9. Business Rules

### 9.1 Catalog

1. Retired courses cannot receive new assignments; existing completions remain historically valid until their own expiry.  
2. Changing validity policy applies to **new** completions by default; retroactive changes require explicit migration command + audit.  
3. Orientations may be project-scoped (site orientation) or company-scoped.

### 9.2 Competencies

1. A competency is Valid only if fulfillment rules are met by non-expired completions/evals.  
2. Competency status is computed—not manually painted—except via controlled waiver.  
3. People profile matrix displays Training-computed status.

### 9.3 Requirements → Assignments

1. Requirements are declarative; assignments are the actionable obligations.  
2. When a person enters scope (trade assigned, project membership granted), Training must create/ensure assignments (via event handlers/workflows).  
3. Removing a requirement cancels open assignments; completions remain.  
4. Duplicate open assignments for same person+course are coalesced.

### 9.4 Evaluations

1. Pass criteria defined on EvaluationDefinition.  
2. Practical evaluations may require evaluator role permission + Signatures seal.  
3. Failed attempts may allow retries per policy; do not auto-create completion.  
4. Evaluation results attach to person; they do not alter Safety activity records.

### 9.5 Completions & Certificates

1. `TrainingCompletion` is authoritative for validity.  
2. Certificate PDF/image stored as Documents/Files refs on completion evidence.  
3. External certificates require evidence + optional verification workflow.  
4. Revocation immediately sets status Expired/Revoked and emits events for eligibility consumers.  
5. Imports must be idempotent and audited.

### 9.6 Expiry Tracking

1. Every completion with finite validity emits expiring/expired via Temporal watchers.  
2. `Expiring` window configurable per course/tenant.  
3. Expired completions create competency gaps and may open RenewalCase automatically if renewable.  
4. Redis/cache may hold reminders state—never expiry authority.

### 9.7 Renewals

1. RenewalCase links prior completion and required re-qualification steps.  
2. Completing renewal records a **new** TrainingCompletion (lineage to prior).  
3. Grace periods are policy-driven; Safety/Equipment decide whether grace allows work (may query Training status including grace flag).

### 9.8 Training Matrix

1. Matrix cells derive from requirements × people in scope × completions.  
2. Rebuildable from events; UI may read projection.  
3. Project matrix filters people via Core membership.  
4. Matrix is not editable spreadsheet truth—edits go through requirements/completions.

### 9.9 Toolbox Library

1. Library items are content; publishing them does not create Safety attendance proof.  
2. Safety talks may reference `ToolboxLibraryItemId` for topic consistency.  
3. Library tags by trade/region for field search.

### 9.10 Eligibility Contribution

```text
GetPersonCompetency(PersonId, ProjectId?) →
  required items + CompetencyStatus per item + overall Ready/Gaps
```

Consumers (Safety, Equipment, Projects) compose with membership, documents acks, fit-for-work, asset readiness.

### 9.11 Waivers

1. Waivers require elevated permission, reason, expiry, audit.  
2. Waivers appear distinctly in matrix (Waived)—not as Valid.  
3. COR exports must show waivers explicitly.

---

## 10. Workflow Integration

| Workflow | Purpose |
| --- | --- |
| `RequirementSyncWorkflow` | On membership/trade changes, ensure assignments |
| `AssignmentReminderWorkflow` | Remind/overdue escalate |
| `EvaluationSignOffWorkflow` | Practical eval multi-party seals |
| `CompletionExpiryWorkflow` | Expiring/expired signals; gap events |
| `RenewalCampaignWorkflow` | Open renewal, assign steps, track completion |
| `OrientationDueWorkflow` | Site orientation before project work gates |
| `ExternalCertVerificationWorkflow` | Optional verification of uploaded certificates |
| `MatrixRebuildWorkflow` | Periodic/on-demand projection rebuild |

### 10.1 Expiry Sequence

```text
TrainingCompletionRecorded(validTo)
  → start CompletionExpiryWorkflow
  → at T-window: TrainingCompletionExpiring → Notifications + RenewalCase optional
  → at validTo: mark expired → TrainingCompletionExpired → CompetencyGapDetected
  → consumers refresh eligibility
  → Core.AuditApi on revoke/waiver paths
```

### 10.2 Assignment on Project Join

```text
Core ProjectMembershipGranted
  → Training handler/workflow
  → match TrainingRequirements for project/trade/role
  → create TrainingAssignments
  → Notifications to worker/supervisor
```

---

## 11. Notifications

| Trigger | Typical audience |
| --- | --- |
| Assignment created | Worker (+ supervisor digest) |
| Assignment overdue | Worker + supervisor escalation |
| Evaluation needs sign-off | Evaluator / worker |
| Completion expiring | Worker + training admin |
| Completion expired / gap | Worker + supervisor + project safety |
| Renewal opened/overdue | Worker + admin |
| Orientation missing on active project | Worker + supervisor |
| Waiver granted/expired | Admins / auditors (policy) |

Training emits events only; Notifications delivers per tenant policy.

---

## 12. Permissions

| Code | Intent |
| --- | --- |
| `training.course.manage` | Courses/orientations catalog |
| `training.competency.manage` | Competency definitions |
| `training.evaluation.manage` | Evaluation definitions |
| `training.evaluation.assess` | Conduct/score evaluations |
| `training.requirement.manage` | Requirements |
| `training.assignment.manage` | Manual assign/cancel |
| `training.assignment.complete_self` | Worker submits evidence / takes eval |
| `training.completion.record` | Record/import completions |
| `training.completion.revoke` | Revoke completions |
| `training.waiver.grant` | Grant waivers |
| `training.renewal.manage` | Renewal campaigns |
| `training.toolbox.manage` | Toolbox library |
| `training.matrix.read` | View matrix |
| `training.reports.read` | Reports |

Scopes: Tenant for catalog; Project for project requirements/matrix; Self for worker assignments.

---

## 13. Public Interfaces & API (Summary)

### 13.1 Interfaces

| Interface | Purpose |
| --- | --- |
| `TrainingQueryApi` | Courses, assignments, completions, matrix |
| `TrainingCompetencyApi` | `GetPersonCompetency(PersonId, ProjectId?)` |
| `TrainingCommandApi` | Assign/record/revoke for workflows |
| `ToolboxLibraryApi` | List/get toolbox content for Safety |

### 13.2 HTTP (Illustrative)

Base: `/api/training`

- `/courses`, `/orientations`, `/competencies`
- `/evaluations/definitions`, `/evaluations/attempts`
- `/requirements`, `/assignments`
- `/completions`, `/renewals`
- `/toolbox-library`
- `/matrix`
- `/reports/...`

All routes: Core AuthN/AuthZ; significant writes audited.

---

## 14. Reporting

| Report | Purpose |
| --- | --- |
| Training currency by project | % Valid vs gaps |
| Expiry calendar | Upcoming expiries |
| Overdue assignments | Supervision |
| Orientation completion | Site access readiness |
| Evaluation pass rates | Program quality |
| Renewal conversion | Expiring → renewed |
| Waiver register | Audit scrutiny |
| Toolbox library usage | Topic adoption (refs from Safety events) |

Heavy portfolio trends → Analytics/ClickHouse via events; enforcement uses OLTP competency API.

---

## 15. Audit Trail

Core-audit required for:

- Requirement changes  
- Completion record/import/revoke  
- Waiver grant/expire  
- Evaluation pass recorded (especially practical)  
- Renewal force-close  
- Catalog retire affecting mandated courses  

Training retains completion lineage for COR provenance (completion id, evidence refs, validity, signature package ids).

---

## 16. Data Ownership

### 16.1 Schema `training` Owns

- Courses, competencies, evaluations  
- Requirements, assignments, completions, waivers  
- Renewal cases  
- Toolbox library content  
- Matrix & reporting projections  

### 16.2 References Only

| Data | Owner |
| --- | --- |
| Materials & certificate binaries | Documents / Core Files |
| Signature evidence | Signatures |
| Person trades/roles profile | People |
| Project membership | Core |
| Toolbox talk attendance proof | Safety |

---

## 17. Integration With Other Modules

| Module | Interaction |
| --- | --- |
| **People** | PersonId target; consumes events for profile competency cards |
| **Projects** | Required training controls; matrix by project membership |
| **Core** | Authz; membership events drive assignment sync; files; audit |
| **Documents** | Course materials; certificate PDFs; ack of training policies |
| **Signatures** | Evaluation/completion attestations |
| **Safety** | Eligibility gates; toolbox library topic refs |
| **Equipment** | Operator competency before high-risk assign/use |
| **COR** | Training evidence mapping & expiry gaps |
| **Notifications** | Assignment/expiry/renewal fan-out |
| **Workflows** | Expiry, renewal, requirement sync |
| **Analytics** | Currency trends |

---

## 18. Offline Support (Initial)

| Allow offline | Online-only |
| --- | --- |
| View assigned training summary (cached) | Catalog admin |
| Start certain evaluations if packaged offline | Requirement management |
| Upload cert intent queued | Waiver grant |
| | Matrix rebuild |

Completions become authoritative only after server accepts command; expiry clocks are server-side.

---

## 19. Anti-Patterns

1. Editing matrix cells as if a spreadsheet SoR  
2. Storing certificate validity only on People profile cards  
3. Treating Safety toolbox attendance as Training completion (unless explicit bridge command policy)  
4. Client-only expiry reminders without Temporal  
5. Waivers indistinguishable from Valid in APIs  
6. Duplicating Documents version control inside Training  
7. Capturing signature strokes in Training  

---

## 20. Success Criteria

Training is correctly designed when:

1. Every mandated competency has a clear requirement → assignment → completion path.  
2. `GetPersonCompetency` is trusted by Safety and Equipment as the training gate.  
3. Expiry and renewals run durably and notify the right people.  
4. The training matrix reflects live truth and rebuilds from events.  
5. Certificates and seals remain correctly owned by Documents/Signatures.  
6. Toolbox library improves talk consistency without replacing Safety proof.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Principal Architecture | Initial Training domain architecture |

---

*End of Training Domain Architecture*
