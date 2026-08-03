# Proven — Documents Domain Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Documents Domain Architecture (DDD) |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Principal Architecture |
| **Audience** | Engineering, Product, Design, Compliance / Document Control |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Domain Model](./DOMAIN_MODEL.md), [Core Domain](./CORE_DOMAIN.md), [Projects Domain](./PROJECTS_DOMAIN.md), [Safety Domain](./SAFETY_DOMAIN.md), [Equipment Domain](./EQUIPMENT_DOMAIN.md), [System Architecture](./SYSTEM_ARCHITECTURE.md), [UX Architecture](../ux/UX_ARCHITECTURE.md), [PRD](../PRD.md) |

---

## 1. Purpose

This document defines the **Documents** bounded context for Proven.

Documents is the **controlled document management** domain of the Construction Compliance Operating System. It owns policies, SWPs, SJPs, engineering drawings, manuals, forms, and templates as governed document families—with version control, assignment, review, approval, acknowledgement, archive, retention, and search—while collaborating with Core Files for bytes and Signatures for seal evidence (including guest and QR signing).

**Architecture only — no implementation.**

---

## 2. Bounded Context

### 2.1 Context Card

| Field | Value |
| --- | --- |
| **Name** | Document Control |
| **Module** | `documents` |
| **Strategic type** | Supporting domain |
| **Product metaphor** | Controlled copy = effective version you can prove people used |
| **System of record for** | Documents, versions, categories (policy/SWP/SJP/drawing/manual/form/template/…), version control metadata, assignments, review/approval state, acknowledgement requests & records, distribution lists, archive state, retention policies & holds, document search indexes metadata (and content search integration), QR signing *subjects* for documents |
| **Not system of record for** | File bytes (Core Files), signature evidence packages (Signatures), Safety activity instances (Safety), Equipment certification *records* (Equipment—may link to document versions), AuthZ (Core), notification delivery (Notifications) |

### 2.2 Context Map

```text
Core (authz, files, audit, settings)
        │
        ▼
┌─────────────────────────────────────────────┐
│                 DOCUMENTS                   │
│  Library · Versions · Review/Approve        │
│  Assign · Ack · Archive · Retain · Search   │
└──────────────────┬──────────────────────────┘
                   │
     Signatures (seal / guest / QR sign)
     Projects (document links) · Safety (SWP/SJP bindings)
     Equipment (manuals/letters) · Training · COR · Analytics
     Notifications · Workflows
```

### 2.3 Ubiquitous Language

| Term | Meaning |
| --- | --- |
| **Document** | Logical controlled document identity (stable across versions) |
| **Document Version** | Immutable published (or draft) revision of content |
| **Effective Version** | Version in force at a point in time |
| **Category** | Policy, SWP, SJP, Engineering Drawing, Manual, Form, Template, SDS, Other |
| **Assignment** | Obligation for people/projects/roles to read/ack/sign a version |
| **Review** | Structured feedback cycle before approval |
| **Approval** | Formal authorize-to-publish decision |
| **Acknowledgement** | Proof a person accepted an effective version |
| **Guest Signing** | External signer completes required assent without full user access |
| **QR Signing** | Scan-to-open signing/ack package for a document version |
| **Archive** | Terminal library state; retained per policy, not for normal use |
| **Retention Policy** | Rules for keep/dispose/legal hold |
| **Template** | Starter document or form skeleton used to create controlled docs/forms |

---

## 3. Responsibilities (Mapped to Ownership)

| Responsibility | Documents owns? | Clarification |
| --- | --- | --- |
| **Policies** | Yes | Category + lifecycle |
| **SWPs** | Yes | Controlled SWP documents |
| **SJPs** | Yes | Controlled SJP documents |
| **Engineering Drawings** | Yes | Drawing documents + revision letters |
| **Manuals** | Yes | Manufacturer/ops manuals as controlled or reference docs |
| **Forms** | Yes (form *documents* / blank forms) | Runnable safety form *instances* remain Safety; blank/controlled form masters live here |
| **Templates** | Yes | Document templates for creating new controlled docs |
| **Version Control** | Yes | Draft → review → approve → publish → supersede |
| **Assignments** | Yes | Read/ack/sign assignments to audiences |
| **Review** | Yes | Review cycles & comments |
| **Digital Signatures** | Requests + binding | **Signatures** owns evidence; Documents subjects versions |
| **Approval Workflows** | Orchestrates | Temporal + Documents approval state |
| **Guest Signing** | Initiates | Signatures + Core guest policy execute seal |
| **QR Signing** | Issues QR subject | Resolves to version + Signatures package |
| **Archive** | Yes | Archive/retire states |
| **Retention Policies** | Yes | Retention classes, holds, disposal eligibility |

---

## 4. Aggregate Roots

| Aggregate | Responsibility |
| --- | --- |
| **Document** | Stable identity, category, access policy, owning org/project scope, status (Active/Archived/Retired) |
| **DocumentVersion** | Immutable content ref, version number, draft/published state, effective period, checksum |
| **DocumentTemplate** | Template definition for creating documents/forms |
| **ReviewCycle** | Review round(s) for a draft version |
| **ApprovalCase** | Approval workflow instance for a version |
| **Assignment** | Requirement that an audience act on a version (read/ack/sign) |
| **AcknowledgementRequest** | Track per-person (or guest) acknowledgement completion for a version |
| **DistributionList** | Named audience lists for reuse |
| **RetentionPolicy** | Retention rules by category/class |
| **LegalHold** | Hold preventing disposal |
| **DocumentQrSignTarget** | QR identity bound to a version + signing purpose |
| **DocumentSearchProjection** | Searchable metadata/content index projection |

> **Document** + **DocumentVersion** are the primary write models. Review/Approval may be modeled as aggregates keyed by `DocumentVersionId` to avoid bloating the version aggregate with concurrent workflow churn.

---

## 5. Entities

### 5.1 Under Document

| Entity | Description |
| --- | --- |
| **AccessRule** | Who may view/manage (mirrors/enhances Core grants; never replaces AuthZ) |
| **DocumentCollectionMembership** | Folders/collections/tags |
| **LinkedProjectRef** | Optional default project associations |
| **CategoryMetadata** | Category-specific attributes (drawing number, SWP code, etc.) |

### 5.2 Under DocumentVersion

| Entity | Description |
| --- | --- |
| **ContentRef** | `FileObjectId` (+ optional preview rendition ids) |
| **ChangeSummary** | What changed vs prior version |
| **SupersedesRef** | Prior `DocumentVersionId` |
| **RenderableMeta** | Page count, MIME, language |
| **FieldDefinition** *(forms/templates)* | Structured fields if version is a fillable master |

### 5.3 Under Review / Approval

| Entity | Parent | Description |
| --- | --- | --- |
| **ReviewerAssignment** | ReviewCycle | Reviewer person + due date |
| **ReviewComment** | ReviewCycle | Threaded comments / resolutions |
| **ApprovalStep** | ApprovalCase | Ordered approvers |
| **ApprovalDecision** | ApprovalCase | Approve/Reject + reason |

### 5.4 Under Assignment / Acknowledgement

| Entity | Parent | Description |
| --- | --- | --- |
| **AssignmentTarget** | Assignment | Person, role, team, project membership, company, guest set |
| **Acknowledgement** | AcknowledgementRequest | Person/guest completion record |
| **SignaturePackageRef** | Acknowledgement | Link to Signatures package when sealed ack required |
| **ReadReceipt** | Assignment | Optional weaker-than-ack tracking |

### 5.5 Under Retention / QR

| Entity | Description |
| --- | --- |
| **RetentionScheduleApplication** | Policy applied to document/version |
| **DisposalCandidate** | Eligible for dispose after review |
| **QrPayloadBinding** | Code → version + purpose (Ack/Sign/View) |

---

## 6. Value Objects

- `DocumentId`, `DocumentVersionId`, `DocumentTemplateId`
- `DocumentCategory` — Policy | SWP | SJP | EngineeringDrawing | Manual | Form | Template | SDS | Certificate | Other
- `DocumentStatus` — DraftLibrary | Active | Archived | Retired
- `VersionState` — Draft | InReview | PendingApproval | Approved | Published | Superseded | Rejected | Withdrawn
- `VersionNumber` (major.minor or sequential—tenant policy)
- `EffectivePeriod` — effectiveFrom, effectiveTo?
- `AssignmentPurpose` — Read | Acknowledge | Sign
- `AcknowledgementStatus` — Pending | Completed | Overdue | Waived | Expired
- `ApprovalOutcome` — Approved | Rejected | Cancelled
- `RetentionClass`, `RetentionDuration`, `DisposalAction`
- `HoldStatus` — None | OnHold
- `Checksum`, `ContentType`, `FileObjectId`
- `GuestSignerRef`, `QrCodeId`, `SignaturePackageId`
- `SearchDocumentRef` (index keys)
- `ProjectId`, `PersonId`, `CompanyId`, `TeamId`

---

## 7. Relationships

```text
Document 1──* DocumentVersion
     │              │
     │              ├── ContentRef ──► FileObject (Core)
     │              ├── ReviewCycle ──► ApprovalCase
     │              ├── Assignment ──► AcknowledgementRequest ──► Acknowledgement
     │              │                      └── SignaturePackage (Signatures)
     │              ├── DocumentQrSignTarget
     │              └── RetentionScheduleApplication ──► RetentionPolicy
     │
     ├── DocumentTemplate (optional source)
     └── LegalHold (optional)

Projects.ProjectDocumentLink ──► Document / DocumentVersion
Safety.SafetyProcedureBinding ──► DocumentVersion (SWP/SJP)
Equipment binder sections / certs ──► DocumentVersion
Training evidence ──► DocumentVersion (optional)
COR evidence mapping ──► DocumentVersion + Acknowledgements
```

### 7.1 Forms vs Safety Forms

```text
Documents Form (master / blank / controlled form document)
        │ used as
        ▼
Safety Activity Type may reference form master DocumentVersionId
        │ runtime instances are
        ▼
SafetyActivity (SoR for filled responses + seals)
```

Documents does **not** store day-to-day completed FLHA payloads.

---

## 8. Document Lifecycle

```text
[Template optional]
      │
      ▼
Create Document + Draft Version
      │
      ▼
Author edits (new draft version or mutate draft content ref)
      │
      ▼
Submit for Review ──► ReviewCycle (comments / resolve)
      │
      ▼
Submit for Approval ──► ApprovalCase (Temporal workflow)
      │
      ├─ Rejected ──► revise new Draft Version
      │
      ▼
Approved
      │
      ▼
Publish Version
      ├─ set EffectivePeriod
      ├─ supersede prior Published version
      ├─ emit DocumentVersionPublished
      └─ optional auto Assign / Ack requests
      │
      ▼
In Force (Effective)
      │
      ├─ Acknowledge / Sign (people, guests, QR)
      ├─ New draft revision cycle…
      │
      ▼
Superseded (still retained)
      │
      ▼
Archive Document (library-level) / Retire
      │
      ▼
Retention → Legal Hold? → Dispose eligibility → Disposed (per policy)
```

### 8.1 Lifecycle Rules

1. Published versions are **immutable**. Fixes require a new version.  
2. Exactly one *current effective* published version per document at time *t* (unless multi-effective policy for bilingual packs—rare; default single).  
3. Publishing must supersede previous effective version atomically in domain terms.  
4. Drafts are not assignable for mandatory acknowledgement unless tenant explicitly allows “draft preview.”  
5. Archived documents cannot be newly assigned; existing historical acks remain valid.  
6. Retired is stronger than archive (no restore without elevated permission).  
7. Effective dating supports future-publish (approve now, effective Monday).

---

## 9. Domain Events

### 9.1 Library & Versions

- `DocumentCreated`
- `DocumentUpdated`
- `DocumentArchived`
- `DocumentRetired`
- `DocumentRestored`
- `DocumentVersionCreated`
- `DocumentVersionUpdated` *(draft only)*
- `DocumentVersionSubmittedForReview`
- `DocumentVersionSubmittedForApproval`
- `DocumentVersionApproved`
- `DocumentVersionRejected`
- `DocumentVersionPublished`
- `DocumentVersionSuperseded`
- `DocumentVersionWithdrawn`

### 9.2 Assignment, Ack, Signing

- `DocumentAssignmentCreated`
- `DocumentAssignmentCancelled`
- `AcknowledgementRequested`
- `DocumentAcknowledged`
- `DocumentAcknowledgementOverdue`
- `DocumentSignatureRequested`
- `DocumentSigningCompleted`
- `GuestSignLinkIssued`
- `DocumentQrSignTargetIssued`
- `DocumentQrSignCompleted`

### 9.3 Retention & Distribution

- `DistributionIssued`
- `RetentionPolicyApplied`
- `LegalHoldApplied`
- `LegalHoldReleased`
- `DocumentDisposalEligible`
- `DocumentDisposed`

### 9.4 Search

- `DocumentSearchProjectionUpdated`

---

## 10. Workflow Integration

| Workflow | Purpose |
| --- | --- |
| `DocumentReviewWorkflow` | Reviewer assignments, due dates, escalate incomplete reviews |
| `DocumentApprovalWorkflow` | Sequential/parallel approval steps; reject/revise loops |
| `DocumentPublishWorkflow` | Optional delayed effective publish; post-publish assignment fan-out |
| `AcknowledgementCampaignWorkflow` | Reminders, overdue, completion metrics |
| `GuestSignWorkflow` | Time-boxed guest link; expiry; completion |
| `QrSignSessionWorkflow` | QR issue → open → sign → complete/expire |
| `RetentionDisposalWorkflow` | Eligibility checks, approvals to dispose, hold conflicts |

### 10.1 Integration Rules

1. Temporal orchestrates; Documents aggregates enforce version immutability and effective-period invariants.  
2. Approval decisions are domain commands invoked by workflow activities.  
3. Signing always creates/completes packages in **Signatures**; Documents records refs + ack state.  
4. Never implement approval timers only in the UI.

### 10.2 Publish Sequence

```text
Approve Version
  → PublishDocumentVersion(effectiveFrom)
  → supersede prior
  → Core.AuditApi
  → events → Projects/Safety/Equipment consumers refresh “current version”
  → optional start AcknowledgementCampaignWorkflow
  → Notifications
```

### 10.3 Guest / QR Signing Sequence

```text
Create Assignment(purpose=Sign) for guest or QR audience
  → Issue GuestSignLink or DocumentQrSignTarget
  → Signatures.CreateSignaturePackage(subject=DocumentVersion)
  → signer seals
  → DocumentAcknowledged / DocumentSigningCompleted
  → audit + notifications
```

Guest UX remains the minimal guest surface from UX architecture; Documents does not become a full portal.

---

## 11. Business Rules (Selected)

### 11.1 Categories

1. Category determines default retention class, required approval strength, and whether acknowledgement is typical (SWP/SJP/Policy often yes; drawings maybe view-only).  
2. Category changes on Active documents require elevated permission and audit.

### 11.2 Versioning

1. Version numbers monotonic per document.  
2. Content checksum required at publish.  
3. Replacing file on Published version is forbidden.  
4. Authors may upload new draft content via Core File upload intent.

### 11.3 Review & Approval

1. Approval graph defined by tenant policy / document category.  
2. Self-approval may be forbidden for SWP/SJP.  
3. Reject returns to Draft with mandatory comment.  
4. Concurrent approval cases per version: one active max.

### 11.4 Assignments & Acknowledgements

1. Assignment targets resolve through Core membership/teams/roles and People directories—not ad hoc email-only lists without identity policy.  
2. Mandatory ack incomplete → eligibility consumers may block (Training/Safety gates query Documents).  
3. Waivers require elevated permission + reason + audit.  
4. Supersede invalidates pending acks on old version; new campaign for new effective version.

### 11.5 Signatures / Guest / QR

1. Documents never stores signature strokes.  
2. Guest signing requires Signatures + Core guest policy; links expire.  
3. QR signing binds to a specific `DocumentVersionId` (not “latest” floating at scan time—snapshot at issue, or explicit “always current” policy flag with careful audit).  
4. Default QR target should pin version for evidence integrity unless campaign type is “always effective.”

### 11.6 Archive & Retention

1. Archive removes from default library search for field users; admins/auditors retain access.  
2. Legal hold blocks dispose regardless of retention elapsed.  
3. Disposal is dual-controlled (policy + workflow approval) and audited.  
4. Retention policy attaches by category defaults overrideable per document.

### 11.7 Cross-Module Effectiveness

1. `GetCurrentEffectiveVersion(DocumentId, atTime)` is the authoritative query for Safety/Equipment/Projects.  
2. Consumers must store `DocumentVersionId` on acknowledgements/evidence—not only `DocumentId`.

---

## 12. Permissions

| Code | Intent |
| --- | --- |
| `documents.document.read` | View metadata + permitted content |
| `documents.document.create` | Create document |
| `documents.document.manage` | Update metadata/collections |
| `documents.version.author` | Create/edit drafts |
| `documents.version.review` | Participate in review |
| `documents.version.approve` | Approve/reject |
| `documents.version.publish` | Publish/withdraw |
| `documents.assignment.manage` | Create campaigns/assignments |
| `documents.ack.complete` | Acknowledge/sign as assignee |
| `documents.guest.issue` | Issue guest sign links |
| `documents.qr.issue` | Issue QR sign targets |
| `documents.archive.manage` | Archive/restore |
| `documents.retention.manage` | Policies/holds/disposal |
| `documents.template.manage` | Templates |
| `documents.search.admin` | Reindex/admin search |
| `documents.reports.read` | Reporting |

Scopes: Tenant library admin vs Project-scoped document sets; field workers typically `read` + `ack.complete` on assigned items.

---

## 13. Audit Trail

Core Audit must record:

- Create/archive/retire  
- Publish/supersede/withdraw  
- Approve/reject  
- Assignment create/cancel  
- Acknowledgement complete / waive  
- Guest/QR issue  
- Legal hold apply/release  
- Dispose  

Documents also keeps version lineage and acknowledgement records as domain evidence for COR. Lineage ≠ security audit substitute.

---

## 14. Public Interfaces & API

### 14.1 In-Process

| Interface | Purpose |
| --- | --- |
| `DocumentsQueryApi` | Get document, effective version, ack status, search |
| `DocumentsCommandApi` | Publish/assign/ack commands for workflows |
| `EffectiveVersionApi` | `GetCurrentEffectiveVersion(DocumentId, atTime)` |
| `AcknowledgementApi` | `IsAcknowledged(PersonId, DocumentVersionId)`, pending lists |

### 14.2 HTTP API (Illustrative)

Base: `/api/documents`

| Area | Paths |
| --- | --- |
| Library | `GET/POST /documents`, `PATCH /documents/{id}` |
| Versions | `POST /documents/{id}/versions`, `POST .../submit-review`, `.../approve`, `.../publish` |
| Templates | `/templates` |
| Assignments | `/assignments`, `/acknowledgements` |
| Guest | `/guest-sign/issues` |
| QR | `/qr-targets`, `GET /qr/{code}` |
| Archive | `POST /documents/{id}/archive` |
| Retention | `/retention-policies`, `/legal-holds`, `/disposals` |
| Search | `GET /search?q=` |
| Reports | `/reports/ack-completion`, `/reports/overdue-acks`, … |

Downloads use Core authorized file access / presigned GET after Documents authz.

---

## 15. Search

### 15.1 Phased Approach

| Phase | Technology | Scope |
| --- | --- | --- |
| Initial | PostgreSQL FTS + metadata filters | Title, code, category, tags, project, text extract where available |
| Later | OpenSearch (when required) | Large tenants, heavy content search |

### 15.2 Search Model

- Index **published** (and optionally draft for authors) versions.  
- Metadata: category, codes, project links, effective dates, status.  
- Content: extracted text from allowed MIME types via async worker (Go) updating Documents projection—**worker does not own ACL**.  
- Every search hit enforces Documents + Core authz before returning content snippets.  
- Archived/retired filtered from default field search.

### 15.3 Events

`DocumentVersionPublished` / `Superseded` / `Archived` → reindex/update projection.

---

## 16. Reporting

| Report | Use |
| --- | --- |
| Acknowledgement completion by project/document | Compliance campaigns |
| Overdue acknowledgements | Supervision / My Actions upstream |
| Documents pending review/approval | Document control workload |
| Effective version inventory | Audit readiness |
| Retention/hold register | Legal/compliance |
| Guest/QR completion rates | Field distribution effectiveness |
| Supersedure lag (draft age) | Process health |

Portfolio trends may flow to Analytics/ClickHouse via events; enforcement queries stay on OLTP effective-version/ack APIs.

---

## 17. Notifications

Documents emits events; Notifications delivers:

- Review/approval requested or overdue  
- Published documents requiring ack  
- Ack overdue escalations  
- Guest link issued (to guest channel)  
- Legal hold / disposal action notices  
- Rejected version to authors  

---

## 18. Data Ownership

### 18.1 Schema `documents` Owns

- Documents, versions, templates  
- Review/approval cases  
- Assignments, acknowledgement requests/records  
- Distribution lists  
- Retention policies, legal holds, disposal records  
- QR sign targets  
- Search projections  

### 18.2 Not Owned

| Concern | Owner |
| --- | --- |
| Bytes in R2 | Core Files |
| Signature strokes/evidence | Signatures |
| Filled FLHA/toolbox instances | Safety |
| Equipment cert *register* | Equipment (links to versions) |
| Project link rows | Projects (plus Documents identity) |

---

## 19. How Other Modules Consume Documents

| Module | Consumption |
| --- | --- |
| **Safety** | SWP/SJP effective versions; procedure ack; bulletin bodies optional |
| **Equipment** | Manuals, engineering letters, cert PDFs as versions in binders |
| **Projects** | Document links & required acknowledgements |
| **Training** | Training material / certificate documents |
| **People** | Certification profile may reference document versions |
| **Signatures** | Subject = DocumentVersion for ack/sign |
| **COR** | Effective policies/SWP evidence + ack provenance |
| **Analytics** | Campaign completion trends |
| **Web/PWA** | Library, My Actions acks, guest/QR flows |

### 19.1 Effective Version Gate

```text
Consumer needs current SWP
  → Documents.GetCurrentEffectiveVersion(docId, now)
  → use DocumentVersionId in evidence
  → optional IsAcknowledged(person, versionId)
```

---

## 20. Anti-Patterns

1. Mutating published files in place  
2. Storing acknowledgements only as “checkbox” without version id  
3. Implementing approvals only in React state  
4. Guest links that never expire  
5. QR always resolving floating “latest” without policy/audit  
6. Search returning hits without authz  
7. Duplicating Safety completed forms inside Documents  
8. Bypassing Core audit on publish/dispose/hold  

---

## 21. Success Criteria

Documents is correctly designed when:

1. Field and office users always know the **effective** controlled copy.  
2. SWP/SJP/policy acknowledgements are version-bound and sealable.  
3. Review/approval/publish are workflow-durable and auditable.  
4. Guest and QR signing produce Signatures-grade evidence without opening the full OS.  
5. Archive/retention/legal hold protect evidence without cluttering daily library UX.  
6. Search is fast and permission-safe; OpenSearch can arrive later without rewriting ownership.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Principal Architecture | Initial Documents domain architecture |

---

*End of Documents Domain Architecture*
