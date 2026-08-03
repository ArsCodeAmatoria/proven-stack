# Proven — Projects Domain Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Projects Domain Architecture (DDD) |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Principal Architecture |
| **Audience** | Engineering, Product, Design |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Domain Model](./DOMAIN_MODEL.md), [Core Domain](./CORE_DOMAIN.md), [System Architecture](./SYSTEM_ARCHITECTURE.md), [UX Architecture](../ux/UX_ARCHITECTURE.md), [PRD](../PRD.md) |

---

## 1. Purpose

This document defines the **Projects** bounded context for Proven.

Projects is the **Place** module of the Construction Compliance Operating System: it defines the construction undertaking, who participates as companies, where work happens, what is required on that site, and the project-scoped operating dashboard.

**Documentation only — no application code.**

---

## 2. Strategic Role

### 2.1 Context Card

| Field | Value |
| --- | --- |
| **Name** | Projects |
| **Module** | `projects` |
| **Strategic type** | Supporting domain |
| **Product metaphor** | Project = Place (job site / undertaking) |
| **System of record for** | Project lifecycle, company participants (prime/sub/client), locations/areas, project settings, required controls, project templates, project document *links*, project form/template *bindings*, dashboard projections owned by Projects |
| **Not system of record for** | Auth membership ACL (Core), person profiles (Workforce), safety records (Safety), asset registry (Equipment), controlled document binaries/versions (Documents), signature evidence (Signatures) |

### 2.2 Why Projects Exists

Compliance work is meaningless without scope. Projects answers:

1. What undertaking exists?  
2. Which companies are Prime, Subcontractors, Client?  
3. Where are the locations/areas?  
4. What controls, forms, documents, and equipment rules are required here?  
5. What is the project’s operational health (dashboard)?  

### 2.3 Boundary With Core

| Concern | Owner |
| --- | --- |
| Project lifecycle & definition | **Projects** |
| Project Membership (who may access / participate as principal/person) | **Core** |
| Teams (team aggregate & member list) | **Core** (project-scoped teams) |
| Tenant/Company master data | **Core** |
| Authorize(`projects.*`, Project scope) | **Core** |

Projects **initiates** membership and team setup by calling Core public commands after validating project invariants. Projects never stores a competing ACL table.

---

## 3. Responsibilities (Mapped to Ownership)

| Responsibility | Projects owns? | Clarification |
| --- | --- | --- |
| **Project lifecycle** | Yes | Create → activate → hold → close → archive |
| **Prime Contractor** | Yes (as participant) | References `CompanyId` from Core |
| **Subcontractors** | Yes (as participants) | Many; each is a company participant |
| **Client** | Yes (as participant) | Client/owner company participant |
| **Locations** | Yes | Site address + areas/zones |
| **Project Teams** | Orchestrates | Team records live in Core; Projects links/requires teams for the place |
| **Safety Statistics** | Projection only | Derived from Safety (and related) events; not authoritative safety data |
| **Project Settings** | Yes | Project-scoped configuration aggregate |
| **Documents** | Links & requirements | Document authority in `documents`; Projects stores library links / required ack refs |
| **Equipment Assignments** | Rules + read models | Assignment authority in `equipment`; Projects declares needs and shows assignment status via queries/events |
| **Worker Assignments** | Orchestrates | Authoritative membership in Core; Projects may hold assignment *preferences* / roster views |
| **Forms** | Bindings & requirements | Form *definitions/instances* for safety live in Safety (or forms capability there); Projects binds required form types to the project |
| **Templates** | Yes | `ProjectTemplate` for repeatable setup |
| **Project Dashboard** | Yes (composition + projections) | Assembles Place overview from owned data + foreign read models |

---

## 4. Ubiquitous Language

| Term | Meaning |
| --- | --- |
| **Project** | Construction undertaking / Place that scopes compliance work |
| **Project Code** | Human-readable identifier within a tenant |
| **Lifecycle Status** | Planning, Active, OnHold, Closed, Archived |
| **Participant** | A Core `Company` engaged on the project in a participation role |
| **Prime Contractor** | Participant with role Prime |
| **Subcontractor** | Participant with role Subcontractor |
| **Client** | Participant with role Client (owner/principal client) |
| **Location** | Primary site locality for the project |
| **Area** | Sub-location / zone within a project (floor, bridge span, yard, etc.) |
| **Required Control** | Project rule referencing another module’s requirement type/id |
| **Project Settings** | Configuration values for this project |
| **Document Link** | Reference to a Documents-module document/version required or filed on the project |
| **Form Binding** | Reference to a form/activity type required on the project |
| **Template** | Reusable blueprint to create projects with defaults |
| **Dashboard Snapshot** | Project-scoped operational projection for Place overview |
| **Proof Health** | Derived indicator of sealed/complete vs open compliance work (projection) |

---

## 5. Aggregate Roots

| Aggregate | Responsibility |
| --- | --- |
| **Project** | Lifecycle, identity, primary location, participants, areas, required controls, document links, form bindings, equipment requirement refs |
| **ProjectSettings** | Project-scoped settings document (typed keys) |
| **ProjectTemplate** | Reusable creation blueprint (participants pattern, controls, settings defaults, form bindings) |
| **ProjectDashboardProjection** | Read-model aggregate/projection store for Place dashboard (updated from events; not a write-side command model for safety) |

> Prefer **Project** as the primary write aggregate. Keep settings separate when settings churn would contend with lifecycle/participant changes. Dashboard projection is updated asynchronously.

---

## 6. Entities

### 6.1 Under Project

| Entity | Description |
| --- | --- |
| **ProjectParticipant** | Company engagement: Prime, Subcontractor, Client, Supplier, Other |
| **ProjectLocation** | Primary site location (may be singular embedded VO if simple; entity if history matters) |
| **ProjectArea** | Named sub-area with optional geo/code |
| **RequiredControl** | Typed reference to an external requirement (safety activity type, training requirement, document ack, equipment rule) |
| **ProjectDocumentLink** | Link to `DocumentId` / `DocumentVersionId` with purpose (RequiredAck, Reference, Contractual, SiteFile) |
| **ProjectFormBinding** | Link to form/activity type required on project (frequency, mandatory flag, owning module type) |
| **EquipmentRequirement** | Rule describing equipment types/checks required on project (refs Equipment module types) |
| **WorkerAssignmentView** *(optional local entity)* | Non-authoritative roster cache keyed by Core membership ids for Place UI (rebuildable) |
| **TeamLink** | Reference to Core `TeamId` associated with this project |

### 6.2 Under ProjectSettings

| Entity | Description |
| --- | --- |
| **SettingEntry** | Key/value within project settings schema |

### 6.3 Under ProjectTemplate

| Entity | Description |
| --- | --- |
| **TemplateParticipantSlot** | Expected participant roles (e.g., must have Prime) |
| **TemplateControl** | Default required controls |
| **TemplateFormBinding** | Default form bindings |
| **TemplateDocumentLink** | Default document requirements (by document template/catalog ref) |
| **TemplateSettings** | Default project settings |

### 6.4 Under Dashboard Projection

| Entity | Description |
| --- | --- |
| **SafetyStatCounters** | Open activities, overdue CAs, sealed today, incidents open (projected) |
| **TrainingGapCounters** | Project-scoped gap counts (projected) |
| **EquipmentReadinessCounters** | Ready / blocked assets on project (projected) |
| **ProofHealthScore** | Derived score/summary for Place header |

---

## 7. Value Objects

### 7.1 Identifiers & Refs

- `ProjectId`, `ProjectCode`
- `ProjectTemplateId`
- `CompanyId` *(Core ref)*
- `TeamId` *(Core ref)*
- `PersonId` *(Workforce ref)*
- `DocumentId`, `DocumentVersionId` *(Documents refs)*
- `FileObjectId` *(Core files ref, if project files use Core directly)*
- `ActivityTypeId` / `FormTypeId` *(Safety or forms catalog ref)*
- `TrainingRequirementId` *(Training ref)*
- `AssetTypeId` / `AssetId` *(Equipment refs)*
- `ControlRef` — `{ module, type, id, params? }`

### 7.2 States & Enums

- `ProjectStatus` — `Planning` | `Active` | `OnHold` | `Closed` | `Archived`
- `ParticipationRole` — `Prime` | `Subcontractor` | `Client` | `Supplier` | `Other`
- `ParticipantStatus` — `Invited` | `Active` | `Suspended` | `Removed`
- `DocumentLinkPurpose` — `RequiredAcknowledgement` | `Reference` | `Contractual` | `SiteFile`
- `FormCadence` — `Once` | `Daily` | `PerShift` | `PerTask` | `OnDemand` | `Custom`
- `AreaStatus` — `Active` | `Inactive`

### 7.3 Structured Values

- `ProjectName`, `ProjectDescription`
- `DateRange` (planned start/end)
- `RegionCode` (aligns with Core/tenant region)
- `SiteAddress`, `GeoCoordinate`
- `Timezone`
- `ProofHealth` — `{ score?, sealedRatio?, openExceptions }`
- `SettingValue` (schema-typed)

---

## 8. Relationships

```text
Tenant (Core)
  └── Project (Projects)
        ├── ProjectParticipant ──► Company (Core)
        │     · exactly one Active Prime (business rule)
        │     · zero/one Active Client (configurable)
        │     · many Subcontractors
        ├── ProjectLocation / ProjectArea
        ├── ProjectSettings (1:1)
        ├── TeamLink ──► Team (Core, project-scoped)
        ├── RequiredControl ──► Safety | Training | Documents | Equipment (by ref)
        ├── ProjectDocumentLink ──► Document/DocumentVersion (Documents)
        ├── ProjectFormBinding ──► Form/Activity types (Safety/Forms catalog)
        ├── EquipmentRequirement ──► Equipment types/rules
        └── DashboardProjection ◄── events from Safety, Training, Equipment, Documents, Core membership

ProjectTemplate ──applies_to_create──► Project
```

### 8.1 Relationship Rules

1. **Participants reference companies; they do not duplicate company profiles.**  
2. **Worker access is Core ProjectMembership**, optionally mirrored in assignment views.  
3. **Equipment on site** is known via Equipment module assignments filtered by `ProjectId`, plus Projects’ requirement rules.  
4. **Documents on project** are links/requirements; binaries and versions stay in Documents/Core Files.  
5. **Teams** are Core aggregates; Projects stores association and uses teams for Place UX (crew boards).

---

## 9. Domain Events

### 9.1 Lifecycle

- `ProjectCreated`
- `ProjectUpdated`
- `ProjectActivated`
- `ProjectPutOnHold`
- `ProjectResumed`
- `ProjectClosed`
- `ProjectArchived`
- `ProjectReopened` *(rare; policy-gated)*

### 9.2 Participants

- `ProjectParticipantAdded`
- `ProjectParticipantUpdated`
- `ProjectParticipantSuspended`
- `ProjectParticipantRemoved`
- `ProjectPrimeAssigned`
- `ProjectPrimeChanged`
- `ProjectClientAssigned`

### 9.3 Locations & Areas

- `ProjectLocationSet`
- `ProjectAreaAdded`
- `ProjectAreaUpdated`
- `ProjectAreaDeactivated`

### 9.4 Controls, Forms, Documents, Equipment Requirements

- `RequiredControlDefined`
- `RequiredControlUpdated`
- `RequiredControlRemoved`
- `ProjectFormBindingAdded`
- `ProjectFormBindingRemoved`
- `ProjectDocumentLinked`
- `ProjectDocumentUnlinked`
- `EquipmentRequirementDefined`
- `EquipmentRequirementRemoved`

### 9.5 Teams & Settings & Templates

- `ProjectTeamLinked`
- `ProjectTeamUnlinked`
- `ProjectSettingsChanged`
- `ProjectTemplateCreated`
- `ProjectTemplatePublished`
- `ProjectTemplateRetired`
- `ProjectCreatedFromTemplate`

### 9.6 Dashboard / Projection (optional integration events)

- `ProjectDashboardRebuilt`
- `ProjectProofHealthChanged`

### 9.7 Envelope

Standard Proven envelope: `tenant_id`, `project_id`, actor, correlation/causation IDs, versioned payload. No foreign aggregate internals.

---

## 10. Commands (Write Model)

| Command | Effect |
| --- | --- |
| `CreateProject` | Create in Planning; optional template apply |
| `UpdateProjectDetails` | Name, code (if allowed), dates, description |
| `ActivateProject` | Planning/OnHold → Active (invariants) |
| `PutProjectOnHold` | Active → OnHold |
| `ResumeProject` | OnHold → Active |
| `CloseProject` | → Closed (block new work policies) |
| `ArchiveProject` | Closed → Archived |
| `AddParticipant` | Add Prime/Sub/Client/… |
| `UpdateParticipant` | Role/status changes with invariants |
| `RemoveParticipant` | Soft-remove; may require membership cleanup via Core |
| `SetProjectLocation` | Primary location |
| `AddProjectArea` / `UpdateProjectArea` / `DeactivateProjectArea` | Areas |
| `DefineRequiredControl` / `RemoveRequiredControl` | Controls |
| `BindForm` / `UnbindForm` | Form requirements |
| `LinkDocument` / `UnlinkDocument` | Document links |
| `DefineEquipmentRequirement` / `RemoveEquipmentRequirement` | Equipment rules |
| `LinkProjectTeam` / `UnlinkProjectTeam` | Assoc to Core team (after Core create) |
| `UpsertProjectSettings` | Settings |
| `CreateProjectTemplate` / `PublishProjectTemplate` / `RetireProjectTemplate` | Templates |
| `RebuildProjectDashboard` | Admin/ops projection rebuild |

**Orchestrating commands (application services, not leaking Core internals):**

| Application command | Projects + Core collaboration |
| --- | --- |
| `AssignWorkerToProject` | Validate project active → `Core.GrantProjectMembership` → local roster projection |
| `UnassignWorkerFromProject` | `Core.RevokeProjectMembership` → projection update |
| `CreateProjectTeam` | `Core.CreateTeam(scope=Project)` → `LinkProjectTeam` |
| `RequestEquipmentAssignment` | Call Equipment public API with `ProjectId` (Equipment owns write) |

---

## 11. Queries

| Query | Returns | Consumers |
| --- | --- | --- |
| `GetProject(ProjectId)` | Project summary + status + primary location | All modules, UI |
| `GetProjectDetail(ProjectId)` | Participants, areas, settings summary | UI Place |
| `ListProjects(filter)` | Tenant project list | UI, Analytics |
| `ListParticipants(ProjectId)` | Company participants | UI, COR, Analytics |
| `GetProjectAreas(ProjectId)` | Areas | Safety/Equipment scoping |
| `ListRequiredControls(ProjectId)` | Controls | Safety, Training, Documents, Equipment |
| `ListFormBindings(ProjectId)` | Required forms | Safety, My Actions planners |
| `ListDocumentLinks(ProjectId)` | Linked docs | Documents UI, workers |
| `ListEquipmentRequirements(ProjectId)` | Requirements | Equipment |
| `GetProjectSettings(ProjectId)` | Settings DTO | All |
| `GetProjectDashboard(ProjectId)` | Place dashboard DTO | Web Command Center / Place Overview |
| `IsProjectActive(ProjectId)` | bool | Gates in other modules |
| `AssertProjectExists(ProjectId)` | exists + tenant match | Foreign writes |
| `ListTemplates(tenant)` | Templates | Admin |
| `GetProofHealth(ProjectId)` | Proof health VO | UI, Analytics |

---

## 12. Business Rules

### 12.1 Lifecycle

1. New projects start in `Planning` unless policy allows direct `Active`.  
2. Only `Active` projects accept new field compliance work by default (Safety/Equipment/Training gates use `IsProjectActive`).  
3. `OnHold` pauses new mandatory field work; existing open items remain visible.  
4. `Closed` projects are read-mostly; new activities blocked; reopen requires elevated permission.  
5. `Archived` is terminal for normal operators; hidden from default lists.  
6. Closing may require warnings (not hard blocks) for open corrective actions—product policy; hard blocks only where compliance policy demands.

### 12.2 Participants

1. A project must have **exactly one Active Prime** before `ActivateProject` (configurable for rare owner-only modes via settings).  
2. Changing Prime emits `ProjectPrimeChanged` and is audited via Core Audit.  
3. Client is optional but unique when present.  
4. Subcontractors may be many; duplicates of the same `CompanyId` in the same role are rejected.  
5. Removing a participant does not delete the Core Company.  
6. Participant company must belong to / be registered in the tenant (Core query).

### 12.3 Locations

1. Active projects should have a primary location before field launch (warning or hard gate via settings).  
2. Areas belong to one project; codes unique per project.  
3. Deactivating an area does not delete historical records referencing it (foreign modules keep AreaId snapshots).

### 12.4 Required Controls, Forms, Documents

1. Required controls store **references**, never copies of foreign business rules.  
2. Form bindings cannot reference unknown form/activity types (validate via Safety/Forms catalog query).  
3. Required acknowledgement document links must point to publishable Documents IDs.  
4. Removing a control does not delete historical completions in other modules.  
5. Template application is snapshot-at-create; later template edits do not mutate existing projects unless an explicit “sync from template” command is added (out of initial scope).

### 12.5 Assignments

1. **Worker assignment authority = Core membership.** Projects must not invent bypass membership.  
2. Assigning workers to a Closed/Archived project is rejected.  
3. **Equipment assignment authority = Equipment module.** Projects may define requirements and display status only.  
4. Team membership authority = Core; Projects only links teams to the Place.

### 12.6 Settings

1. Settings keys are schema-registered (timezone, required prime rule, hold behavior, dashboard preferences, etc.).  
2. Settings cannot disable Core AuthZ or license enforcement.  
3. Sensitive setting changes are audited.

### 12.7 Dashboard / Safety Statistics

1. Safety statistics on the dashboard are **projections**, not editable facts.  
2. Source of truth for incidents/activities remains Safety.  
3. If projection lags, UI must allow “as of” freshness or rebuild—never silent invention.  
4. Proof health is descriptive, not a substitute for eligibility decisions.

### 12.8 Multi-Party Visibility

1. Participant role affects UX defaults; **enforcement remains Core grants + membership**.  
2. Subcontractor users see project data only if Core membership/grants allow—Projects does not implement a second permission system.

---

## 13. Permissions

Permissions are registered in Core’s catalog and enforced via `Core.AuthzApi`.

### 13.1 Permission Codes

| Code | Intent |
| --- | --- |
| `projects.project.read` | View project |
| `projects.project.create` | Create project |
| `projects.project.update` | Update details/location/areas |
| `projects.project.activate` | Activate / resume |
| `projects.project.hold` | Put on hold |
| `projects.project.close` | Close |
| `projects.project.archive` | Archive |
| `projects.participant.manage` | Add/update/remove companies |
| `projects.controls.manage` | Required controls / form bindings |
| `projects.documents.link` | Link/unlink documents |
| `projects.settings.manage` | Project settings |
| `projects.template.manage` | Templates |
| `projects.dashboard.read` | View dashboard |
| `projects.assignment.orchestrate` | Trigger worker assign via Core / equipment request |

### 13.2 Typical Scope

- Most permissions evaluated at **Project scope** (or Tenant for create/list).  
- Template management often **Tenant scope**.  
- Worker self-read of assigned projects relies on Core membership + `projects.project.read`.

### 13.3 Enforcement Pattern

```text
Command → Core.Authorize(permission, ProjectScope)
       → Core.IsModuleEnabled("projects")
       → Core.IsProjectMember? (when required)
       → Projects invariants
       → persist + events
       → Core.AuditApi.Append
```

---

## 14. Public Interfaces

In-process interfaces published by `projects` for other modules:

| Interface | Responsibility |
| --- | --- |
| `ProjectQueryApi` | Existence, status, active check, summary, areas |
| `ProjectParticipantsApi` | List participants / prime / subs / client |
| `ProjectRequirementsApi` | Required controls, form bindings, equipment requirements, document links |
| `ProjectSettingsApi` | Read project settings |
| `ProjectDashboardApi` | Read dashboard snapshot |
| `ProjectCommandApi` | Limited commands callable from workflows (e.g., activate after provisioning) |

Other modules **must not** depend on Projects infrastructure/SQL.

---

## 15. HTTP API (Illustrative)

Base: `/api/projects`

| Method | Path | Purpose |
| --- | --- | --- |
| `POST` | `/projects` | Create |
| `GET` | `/projects` | List |
| `GET` | `/projects/{id}` | Detail |
| `PATCH` | `/projects/{id}` | Update details |
| `POST` | `/projects/{id}/activate` | Activate |
| `POST` | `/projects/{id}/hold` | Hold |
| `POST` | `/projects/{id}/close` | Close |
| `POST` | `/projects/{id}/archive` | Archive |
| `GET/POST` | `/projects/{id}/participants` | List/add participants |
| `PATCH/DELETE` | `/projects/{id}/participants/{participantId}` | Update/remove |
| `PUT` | `/projects/{id}/location` | Set location |
| `GET/POST` | `/projects/{id}/areas` | Areas |
| `GET/PUT` | `/projects/{id}/settings` | Settings |
| `GET/POST` | `/projects/{id}/controls` | Required controls |
| `GET/POST` | `/projects/{id}/forms` | Form bindings |
| `GET/POST` | `/projects/{id}/documents` | Document links |
| `GET/POST` | `/projects/{id}/equipment-requirements` | Equipment requirements |
| `POST` | `/projects/{id}/workers` | Orchestrate Core membership grant |
| `DELETE` | `/projects/{id}/workers/{personId}` | Orchestrate revoke |
| `GET` | `/projects/{id}/dashboard` | Place dashboard |
| `GET/POST` | `/templates` | Project templates |

All routes authenticate via Core; authorize via Core.

---

## 16. Data Ownership

### 16.1 Owned by Projects (PostgreSQL schema `projects`)

- Projects, participants, locations, areas  
- Required controls, form bindings, document links, equipment requirements  
- Team links (IDs only)  
- Project settings  
- Project templates  
- Dashboard projection tables  

### 16.2 Referenced, Not Owned

| Data | Owner |
| --- | --- |
| `TenantId`, `CompanyId`, `UserId`, `TeamId`, membership | Core |
| `PersonId`, trades, employment | Workforce |
| Safety activities, CAs, incidents, stats source events | Safety |
| Assets, inspections, assignments | Equipment |
| Documents, versions, acknowledgements | Documents |
| Signature packages | Signatures |
| Training requirements/completions | Training |
| File bytes | Core FileObject / R2 |

### 16.3 Ownership Diagram

```text
┌──────────────────────── Projects Write Models ─────────────────────────┐
│ Project · Participants · Locations · Settings · Templates · Requirements│
│ Document links · Form bindings · Team links · Dashboard projections     │
└───────────────────────────────┬─────────────────────────────────────────┘
                                │ IDs / events / queries only
        ┌───────────────────────┼───────────────────────┐
        ▼                       ▼                       ▼
   Core (ACL, teams,     Safety/Training/         Documents/
   companies, files)     Equipment (facts)        Signatures
```

---

## 17. How Other Modules Consume Project Information

### 17.1 Universal Consumption Patterns

1. **Sync query** — `AssertProjectExists` / `IsProjectActive` / `ListRequiredControls` before accepting commands.  
2. **Store `ProjectId`** on foreign aggregates as a reference.  
3. **Subscribe to Projects events** to update local projections (e.g., invalidate caches when project held).  
4. **Never join** `projects` tables from foreign module SQL.

### 17.2 Per-Module Guide

| Module | How it consumes Projects |
| --- | --- |
| **Core** | Does not depend on Projects for authz catalog; accepts `ProjectId` as scope target when membership/grants created. May query Projects only to validate scope existence if desired (or trust caller + audit). |
| **Workforce** | Uses `ProjectId` when showing assignments; person eligibility still workforce/training-owned. Consumes participant company context for contractor engagement UX. |
| **Safety** | **Required:** active project check; load form bindings & required controls; scope activities to `ProjectId` + optional `AreaId`; publish events that Projects dashboard consumes. |
| **Equipment** | Validates project for assignment; reads equipment requirements; emits assignment/readiness events for dashboard counters. |
| **Documents** | Uses document links & required ack controls; may query participants for distribution defaults; emits ack events for proof health. |
| **Signatures** | Includes `ProjectId` in signature subject context when signing project-scoped records. |
| **Training** | Reads required training controls for project; scopes gap views; events feed dashboard. |
| **COR Audit** | Uses project list/participants as evidence organization dimension; consumes dashboard/proof health as non-authoritative aids. |
| **Notifications** | Routes project-scoped notifications using membership from Core + project names from Projects query. |
| **Workflows** | Project onboarding/activation templates; gates activities on `IsProjectActive`. |
| **Analytics** | Uses project dimensions from events (`ProjectCreated`, status changes, proof health changes); heavy stats from ClickHouse fed by Safety/Training/Equipment events tagged with `ProjectId`. |
| **Web UX** | Place nav and Command Center use `GetProjectDashboard` + Core My Actions; project switcher uses `ListProjects` filtered by Core membership. |

### 17.3 Safety Statistics Flow

```text
Safety domain events (activity closed, CA overdue, incident opened…)
        │
        ▼
NATS → Projects dashboard projector (and/or Analytics)
        │
        ▼
ProjectDashboardProjection.SafetyStatCounters
        │
        ▼
GET /projects/{id}/dashboard  (Place Overview)
```

Projects **does not** recalculate safety outcomes; it **aggregates published facts**.

### 17.4 Worker Assignment Flow

```text
UI / API: Assign worker
  → Projects app service
  → Core.Authorize + Projects.IsProjectActive
  → Core.GrantProjectMembership
  → Core event ProjectMembershipGranted
  → Projects roster projection updates
  → Notifications / Training / Safety consumers react
```

### 17.5 Equipment Assignment Flow

```text
UI: Assign asset to project
  → Equipment.AssignToProject(ProjectId)
  → Equipment validates via Projects.AssertProjectExists / IsProjectActive
  → Equipment event AssetAssignedToProject
  → Projects dashboard equipment counters update
```

### 17.6 Document Link Flow

```text
UI: Require SWP on project
  → Projects.LinkDocument(RequiredAcknowledgement)
  → event ProjectDocumentLinked
  → Documents may create AcknowledgementRequest for membership set
  → completions remain in Documents
```

---

## 18. Project Dashboard Composition

Aligns with UX **Project Place Overview**.

| Block | Data source |
| --- | --- |
| Header (name, status, region, prime) | Projects write model |
| Needs attention | Projection + links into My Actions filters |
| Proof health | Projection |
| People on site counts | Core membership counts (+ Workforce snapshots) |
| Equipment ready | Equipment projections |
| Safety statistics | Safety event projections |
| Training gaps | Training projections |
| Today on this site | Mixed feed (Activity / Safety), permission-scoped |

Dashboard API returns a **DTO assembled for UI**, not a license for other modules to treat counters as source of truth.

---

## 19. Templates

### 19.1 Purpose

Accelerate consistent project setup across regions/trades.

### 19.2 Apply Behavior

`CreateProjectFromTemplate`:

1. Create Project in Planning  
2. Copy template controls, form bindings, document requirement refs, settings defaults  
3. Create participant *slots* guidance (actual companies selected by user)  
4. Emit `ProjectCreatedFromTemplate`  
5. Optionally start onboarding workflow (Teams, default memberships)

Templates do not auto-invent Prime company.

---

## 20. Consistency & Workflows

| Concern | Approach |
| --- | --- |
| Activate project | Sync invariants in Projects; Temporal onboarding for teams/docs distribution optional |
| Membership changes | Core transaction; Projects projection eventual |
| Dashboard stats | Eventual; rebuild command available |
| Hold/Close gating | Foreign modules sync-query `IsProjectActive` on write |
| Template publish | Sync in Projects; consumers read on create only |

---

## 21. Anti-Patterns

1. Storing worker ACL only in Projects  
2. Duplicating company legal profiles in participants  
3. Embedding SafetyActivity records in Projects schema  
4. Treating dashboard counters as authoritative for audits  
5. SQL joins from Safety → Projects tables  
6. Using Redis as permanent project store  
7. Putting business closure rules only in React  

---

## 22. Success Criteria

The Projects module is correct when:

1. Every compliance record can point at a real `ProjectId` Place.  
2. Prime/Sub/Client relationships are clear and validated.  
3. Other modules gate work on project lifecycle without importing Projects internals.  
4. Membership and teams remain Core-owned.  
5. Documents/equipment/safety facts remain in their modules; Projects links and projects requirements only.  
6. Place Dashboard feels like Basecamp-style clarity while showing compliance proof health.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Principal Architecture | Initial Projects domain design aligned with Core |

---

*End of Projects Domain Architecture*
