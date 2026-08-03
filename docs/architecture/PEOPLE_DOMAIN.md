# Proven — People Domain Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | People Domain Architecture (DDD) |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Principal Architecture |
| **Audience** | Engineering, Product, Design, Security |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Domain Model](./DOMAIN_MODEL.md), [Core Domain](./CORE_DOMAIN.md), [Projects Domain](./PROJECTS_DOMAIN.md), [System Architecture](./SYSTEM_ARCHITECTURE.md), [UX Architecture](../ux/UX_ARCHITECTURE.md), [PRD](../PRD.md) |

---

## 1. Purpose

This document defines the **People** bounded context for Proven.

People is the **human system of record** for the Construction Compliance Operating System: who workers, supervisors, and managers are; how they relate to companies; emergency contacts; medical restrictions; availability and attendance; and the profile surfaces that assemble training, certifications, assignments, signatures, and history—without stealing ownership from Training, Core, Signatures, or Notifications.

**Documentation only — no application code.**

### 1.1 Module Naming

| Name | Usage |
| --- | --- |
| **People** | Product / UX language (Directory, profiles) |
| **Module id** | `people` |
| **Prior map alias** | `workforce` in [Domain Model](./DOMAIN_MODEL.md) — superseded by `people` going forward |

---

## 2. Strategic Role

### 2.1 Context Card

| Field | Value |
| --- | --- |
| **Name** | People |
| **Module** | `people` |
| **Strategic type** | Supporting domain |
| **Product metaphor** | Person = human identity in the compliance OS (not a login account) |
| **System of record for** | Person profiles, role classifications (worker/supervisor/manager as workforce roles), emergency contacts, medical restrictions, employments/engagements, trades, availability, attendance (workforce time/presence), competency *profile projections*, certification *profile refs*, assignment *views*, signature *history refs*, notification preference *links*, person history timeline |
| **Not system of record for** | User login/credentials (Core), project ACL membership (Core), training course catalog & completions authority (Training), signature evidence packages (Signatures), notification delivery rules engine (Notifications), safety activity attendance as compliance evidence (Safety) |

### 2.2 Core Distinction

```text
User (Core)     = who can authenticate
Person (People) = who they are operationally on sites
```

Linked via Core `LinkUserToPerson` / People acknowledgment of `UserId` binding. Safety, Training, and Equipment reference **`PersonId`**. AuthZ uses **`PrincipalId` / `UserId`**.

---

## 3. Responsibilities (Mapped to Ownership)

| Responsibility | People owns? | Clarification |
| --- | --- | --- |
| **Workers** | Yes (classification + profile) | Person with workforce role Worker |
| **Supervisors** | Yes (classification + profile) | Workforce role Supervisor (distinct from Core RBAC role, though often paired) |
| **Managers** | Yes (classification + profile) | Workforce role Manager / PM-aligned profile tags |
| **Emergency Contacts** | Yes | Under Person; highly sensitive |
| **Training** | Profile + orchestration | **Training module** owns courses, requirements, completions; People shows status and may deep-link |
| **Competencies** | Projection / matrix view | Derived from Training (+ role/trade rules); People may store matrix *read models* |
| **Certifications** | Refs + profile cards | Evidence/completions in Training/Documents; People holds certification profile entries referencing foreign IDs |
| **Medical Restrictions** | Yes | Sensitive health/limitation records with strict ACL |
| **Project Assignments** | View + orchestration | **Core ProjectMembership** is authority; People projects assignment history/views |
| **Availability** | Yes | Shift/availability declarations and status |
| **Attendance** | Yes (workforce attendance) | Clock/presence records; **not** a substitute for Safety toolbox attendance evidence |
| **Digital Signatures** | History refs only | **Signatures** owns packages/evidence; People lists signature participation history by ref |
| **Notification Preferences** | Link / thin mirror optional | **Notifications** (or Core settings) owns preferences; People profile surfaces entry point |
| **History** | Yes (person timeline) | Append-oriented profile history + foreign event projections |

---

## 4. Ubiquitous Language

| Term | Meaning |
| --- | --- |
| **Person** | Human known to the compliance OS |
| **Workforce Role** | Operational classification: Worker, Supervisor, Manager, Visitor, Temporary (profile-level—not Core permission grants) |
| **Employment** | Employee relationship to a Company |
| **Contractor Engagement** | Person engaged via a contracting Company |
| **Trade** | Trade/discipline code assigned to a person |
| **Emergency Contact** | Contact to notify in emergency |
| **Medical Restriction** | Health/fit-for-work limitation affecting eligibility signals |
| **Competency Profile** | Assembled view of capability status for a person |
| **Certification Profile Entry** | Profile card pointing at a certification/completion/document |
| **Availability** | When a person is available to be scheduled/assigned |
| **Attendance Record** | Workforce presence/time record for a day/shift/project |
| **Assignment View** | Non-authoritative projection of project membership |
| **Signature History Item** | Reference to a sealed signature package the person participated in |
| **Person History** | Timeline of significant person-domain and projected foreign events |

---

## 5. Aggregate Roots

| Aggregate | Responsibility |
| --- | --- |
| **Person** | Profile identity, workforce roles, contact info, status, user link ref, trades, emergency contacts, medical restrictions, certification profile entries |
| **Employment** | Employment lifecycle with a company (may be child entity under Person if churn is low; prefer separate aggregate when concurrent HR updates are common) |
| **ContractorEngagement** | Contractor relationship lifecycle |
| **AvailabilityCalendar** | Availability windows / exceptions for a person |
| **AttendanceRecord** | A single attendance/presence instance (or daily aggregate) |
| **CompetencyProfileProjection** | Rebuildable competency matrix projection for a person |
| **PersonHistoryProjection** | Timeline projection for profile History tab |
| **Crew** *(optional)* | Supervisor-oriented grouping; prefer Core **Team** when team is ACL-relevant—Crew remains People-owned only if purely operational and non-ACL |

> **Recommendation:** Use Core **Teams** for ACL-relevant project crews. Keep People **Crew** only if a distinct non-authz operational roster is required; otherwise link to `TeamId` and avoid duplicate group models.

---

## 6. Entities

### 6.1 Under Person

| Entity | Description |
| --- | --- |
| **WorkforceRoleAssignment** | Worker / Supervisor / Manager / Visitor / Temporary tags (time-bounded) |
| **TradeAssignment** | Trade codes with optional primary flag |
| **EmergencyContact** | Name, relationship, phones; priority order |
| **MedicalRestriction** | Restriction type, severity, effective period, notes ref, work implications flags |
| **CertificationProfileEntry** | Display card: type, issuer snapshot, `TrainingCompletionId` / `DocumentId` / `FileObjectId` refs, expiry |
| **ContactChannel** | Email/phone channels for operational contact (not auth credentials) |
| **PersonUserLink** | Optional local mirror of `UserId` binding (Core remains authority for account) |

### 6.2 Under Employment / Engagement

| Entity | Description |
| --- | --- |
| **EmploymentTerm** | Title, department/org unit ref, start/end |
| **EngagementTerm** | Contracting company, site defaults, period |

### 6.3 Under Availability

| Entity | Description |
| --- | --- |
| **AvailabilityWindow** | Recurring or dated available ranges |
| **AvailabilityException** | PTO, unavailable, modified hours |

### 6.4 Under Attendance

| Entity | Description |
| --- | --- |
| **AttendancePunch** *(optional)* | In/out events if fine-grained |
| **AttendanceAdjustment** | Supervisor correction with reason |

### 6.5 Under Projections

| Entity | Description |
| --- | --- |
| **AssignmentViewItem** | ProjectId, membership roles, status, joined_at (from Core events) |
| **TrainingStatusItem** | Course/requirement status (from Training events) |
| **CompetencyCell** | Dimension × status for matrix |
| **SignatureHistoryItem** | SignaturePackageId, subject ref, sealed_at |
| **HistoryEntry** | Timeline item with source module + ref |

---

## 7. Value Objects

### 7.1 Identifiers

- `PersonId`
- `UserId` *(Core ref)*
- `CompanyId` *(Core ref)*
- `ProjectId` *(Projects ref)*
- `TeamId` *(Core ref)*
- `EmploymentId`, `EngagementId`
- `AttendanceRecordId`, `MedicalRestrictionId`
- `TrainingCompletionId`, `DocumentId`, `SignaturePackageId` *(foreign refs)*

### 7.2 Profile Values

- `PersonName`, `PreferredName`
- `ContactInfo` (email, phone)
- `LocalePreference` *(display; notification prefs owned elsewhere)*
- `WorkforceRoleType` — Worker | Supervisor | Manager | Visitor | Temporary
- `PersonStatus` — Active | Inactive | DeceasedBlocked | Archived
- `TradeCode`
- `EmploymentStatus` — Active | OnLeave | Terminated
- `EngagementStatus` — Active | Completed | Cancelled
- `RestrictionSeverity` — Info | Limit | Block
- `FitForWorkSignal` — Fit | Restricted | NotFit | Unknown
- `AvailabilityStatus` — Available | Limited | Unavailable
- `AttendanceStatus` — Present | Absent | Late | Excused | Partial
- `CertificationStatus` — Valid | Expiring | Expired | Missing | Revoked

### 7.3 Sensitive Handling Markers

- `SensitivityClass` — Standard | PII | Health  
- `RedactionPolicy` — controls query DTO shape by permission  

---

## 8. Relationships

```text
Tenant (Core)
  └── Person (People)
        ├── PersonUserLink ──────────────► User (Core)
        ├── WorkforceRoleAssignment
        ├── TradeAssignment
        ├── EmergencyContact
        ├── MedicalRestriction ──► influences FitForWorkSignal
        ├── CertificationProfileEntry ──► Training / Documents / Files
        ├── Employment ──────────────────► Company (Core)
        ├── ContractorEngagement ────────► Company (Core)
        ├── AvailabilityCalendar
        ├── AttendanceRecord ─(optional)► Project (Projects)
        ├── CompetencyProfileProjection ◄── Training events
        ├── AssignmentView ◄────────────── Core ProjectMembership events
        ├── SignatureHistory ◄──────────── Signatures events
        └── PersonHistory ◄─────────────── People + foreign events

Safety / Equipment / Training aggregates reference PersonId
Core AuthZ uses User/Principal; membership binds PersonId on project
```

### 8.1 Role Relationships (Conceptual)

| Profile role | Typical pairing |
| --- | --- |
| Worker | Field My Actions; Core role Worker |
| Supervisor | Crew direction; Core role Supervisor + project scope |
| Manager | Project/operations oversight; Core grants at project/tenant |

People **workforce roles do not grant permissions**. Core grants do.

---

## 9. Aggregate / Boundary Rules With Other Modules

| Concern | Authority | People responsibility |
| --- | --- | --- |
| Login & sessions | Core | Store/link `UserId`; never passwords |
| Project access | Core membership | Project assignment views; orchestrate assign UX via Projects/Core |
| Teams/crews (ACL) | Core Teams | Display membership; optional operational Crew if needed |
| Training completions | Training | Show status; certification cards by ref |
| Competency rules | Training (+ project required controls) | Matrix projection |
| Controlled cert documents | Documents | Refs only |
| Signature evidence | Signatures | History refs |
| Notification preferences | Notifications module | Profile entry point / deep link; do not fork preference SoR |
| Safety toolbox attendance | Safety | Do not treat People attendance as sealed safety evidence |
| Medical data | People | Strict permissions; minimize event payloads |

---

## 10. Domain Events

### 10.1 Person Lifecycle

- `PersonRegistered`
- `PersonUpdated`
- `PersonActivated`
- `PersonDeactivated`
- `PersonArchived`
- `PersonLinkedToUser`
- `PersonUnlinkedFromUser`
- `WorkforceRoleAssigned`
- `WorkforceRoleRemoved`
- `TradeAssigned`
- `TradeRemoved`

### 10.2 Emergency & Medical

- `EmergencyContactAdded`
- `EmergencyContactUpdated`
- `EmergencyContactRemoved`
- `MedicalRestrictionRecorded`
- `MedicalRestrictionUpdated`
- `MedicalRestrictionCleared`
- `FitForWorkSignalChanged`

> Medical event payloads must **minimize PHI**—prefer IDs + coarse signal (`Restricted`/`NotFit`) for downstream consumers; detailed notes stay query-gated.

### 10.3 Employment & Engagement

- `EmploymentStarted`
- `EmploymentUpdated`
- `EmploymentEnded`
- `ContractorEngagementStarted`
- `ContractorEngagementUpdated`
- `ContractorEngagementEnded`

### 10.4 Availability & Attendance

- `AvailabilityUpdated`
- `AvailabilityExceptionSet`
- `AttendanceRecorded`
- `AttendanceCorrected`
- `AttendanceVoided`

### 10.5 Profile Cards & Projections

- `CertificationProfileEntryAdded`
- `CertificationProfileEntryUpdated`
- `CertificationProfileEntryRemoved`
- `CompetencyProfileRebuilt`
- `PersonAssignmentViewUpdated`
- `PersonSignatureHistoryAppended`
- `PersonHistoryAppended`

### 10.6 Envelope

Standard Proven envelope with `tenant_id`, `person_id`, actor, correlation IDs. Health details redacted in integration events by default.

---

## 11. Commands

| Command | Aggregate | Notes |
| --- | --- | --- |
| `RegisterPerson` | Person | Creates profile |
| `UpdatePersonProfile` | Person | Non-sensitive fields |
| `DeactivatePerson` / `ActivatePerson` | Person | |
| `AssignWorkforceRole` / `RemoveWorkforceRole` | Person | |
| `AssignTrade` / `RemoveTrade` | Person | |
| `AddEmergencyContact` / `UpdateEmergencyContact` / `RemoveEmergencyContact` | Person | |
| `RecordMedicalRestriction` / `UpdateMedicalRestriction` / `ClearMedicalRestriction` | Person | Heightened authz + audit |
| `AddCertificationProfileEntry` / `RemoveCertificationProfileEntry` | Person | Refs must validate against Training/Documents when claimed |
| `StartEmployment` / `EndEmployment` | Employment | Company must exist (Core) |
| `StartContractorEngagement` / `EndContractorEngagement` | ContractorEngagement | |
| `SetAvailability` / `SetAvailabilityException` | AvailabilityCalendar | |
| `RecordAttendance` / `CorrectAttendance` / `VoidAttendance` | AttendanceRecord | |
| `RebuildCompetencyProfile` | Projection | From Training facts |
| `RebuildPersonHistory` | Projection | Ops/admin |

**Orchestrating (application) commands:**

| Command | Collaboration |
| --- | --- |
| `InvitePersonAsUser` | People profile exists → Core `InviteUser` + link |
| `RequestProjectAssignment` | People/Projects UX → Core `GrantProjectMembership` |
| `OpenNotificationPreferences` | Navigate/call Notifications API for SoR prefs |

---

## 12. Queries / Interfaces

### 12.1 Public Query API (`PeopleQueryApi`)

| Query | Result | Consumers |
| --- | --- | --- |
| `GetPerson(PersonId)` | Safe profile DTO | All modules/UI |
| `GetPersonSensitive(PersonId)` | Includes emergency/medical per authz | Safety/Admin restricted |
| `FindPeople(filter)` | Directory search | UI |
| `GetFitForWorkSignal(PersonId)` | Coarse signal | Safety, Projects eligibility composition |
| `ListTrades(PersonId)` | Trades | Training requirements matching |
| `ListEmployments(PersonId)` | Employment DTOs | Admin |
| `GetAvailability(PersonId, range)` | Availability | Scheduling UX |
| `GetAttendance(PersonId, range)` | Attendance list | Supervisors |
| `GetCompetencyProfile(PersonId, ProjectId?)` | Matrix DTO | Supervisors, Safety |
| `GetCertificationProfile(PersonId)` | Cert cards | UI, COR aids |
| `ListAssignmentViews(PersonId)` | Projects assigned (projection) | UI |
| `GetPersonHistory(PersonId)` | Timeline | UI |
| `AssertPersonActive(PersonId)` | bool | Foreign writes |
| `ResolvePersonByUserId(UserId)` | PersonId? | API edge |

### 12.2 Public Command Facade (`PeopleCommandApi`)

Limited commands exposed to workflows/other modules (e.g., `AssertPersonActive` gate helpers, attendance record from approved integrations). Most writes remain People HTTP API.

### 12.3 What People Publishes for Eligibility

People contributes **signals**, not final eligibility:

```text
Eligibility partial (People) =
  PersonActive
  + FitForWorkSignal (coarse)
  + (optional) trade presence
```

Final eligibility still composes Core membership + Training + Documents + Equipment ([Domain Model](./DOMAIN_MODEL.md)).

---

## 13. Permissions

Registered in Core catalog; enforced via `Core.AuthzApi`.

| Code | Intent |
| --- | --- |
| `people.person.read` | View standard profile |
| `people.person.create` | Register person |
| `people.person.update` | Update profile |
| `people.person.deactivate` | Deactivate |
| `people.emergency.read` | View emergency contacts |
| `people.emergency.manage` | Manage emergency contacts |
| `people.medical.read` | View medical restrictions |
| `people.medical.manage` | Record/clear restrictions |
| `people.employment.manage` | Employment/engagement admin |
| `people.availability.manage` | Self or supervisor manage availability |
| `people.attendance.record` | Record attendance |
| `people.attendance.correct` | Supervisor corrections |
| `people.competency.read` | View competency profile |
| `people.history.read` | View person history |
| `people.directory.search` | Search directory |

### 13.1 Sensitivity Rules

1. `people.medical.*` and `people.emergency.*` are **separately granted**—not implied by `people.person.read`.  
2. Self-access policies may allow limited self-read/update without medical write.  
3. All medical/emergency reads and writes append Core audit entries.  
4. Subcontractor visibility limited by Core project membership and company affiliation.

---

## 14. Business Rules

### 14.1 Person Lifecycle

1. Every operational actor on a site should have a `Person` before receiving compliance assignments.  
2. Deactivated persons cannot be newly assigned (Core membership orchestration must check `AssertPersonActive`).  
3. Archiving is soft; historical `PersonId` refs remain valid for evidence.  
4. A Person may exist without a User (badge-only / provisioning pending).  
5. A User should link to at most one active Person per tenant.

### 14.2 Workforce Roles vs Permissions

1. Assigning Workforce Role **Supervisor** does not grant Core permissions.  
2. Admins must grant Core roles/memberships separately (or via onboarding workflow that does both).  
3. UI may suggest pairings but must not conflate the models.

### 14.3 Employment & Companies

1. Employment/engagement requires valid Core `CompanyId`.  
2. Overlapping Active employments with conflicting primary company may be restricted by tenant settings.  
3. Ending employment does not auto-revoke project membership—workflow may propose revocation.

### 14.4 Emergency Contacts

1. At least one emergency contact recommended before Active field deployment (warning or hard gate via settings).  
2. Emergency contacts are not Users by default.  
3. Exports of emergency contacts are tightly permissioned.

### 14.5 Medical Restrictions

1. Medical details are Health-sensitivity class; default event fan-out uses coarse `FitForWorkSignal` only.  
2. `NotFit` / `Restricted` must be visible to supervisors making assignment decisions **if permitted**.  
3. Clearing a restriction requires `people.medical.manage` and audit reason.  
4. People never stores full clinical records—only fit-for-work restrictions relevant to site safety.  
5. Medical restrictions **inform** eligibility; they do not replace Training or Equipment readiness.

### 14.6 Training, Competencies, Certifications

1. People must not invent completion records.  
2. Certification profile entries that claim a Training completion must reference a real `TrainingCompletionId`.  
3. Competency matrix rebuild is idempotent from Training (+ optional Documents) events.  
4. Expiry display in People is informational; Training owns expiry workflows and authoritative status.

### 14.7 Project Assignments

1. Authoritative assignment = Core `ProjectMembership`.  
2. People assignment views are rebuildable projections.  
3. History of assignments is retained for the Person History timeline.  
4. Assigning to Closed projects is rejected by Projects/Core orchestration—not overridden by People.

### 14.8 Availability & Attendance

1. Availability is declarative and does not alone block Core membership (may block scheduling UX).  
2. Attendance corrections require reason + permission.  
3. People attendance is **not** signature-sealed safety evidence.  
4. Safety module attendance/acknowledgements remain required for toolbox/FLHA proof.

### 14.9 Digital Signatures

1. People does not capture signature strokes or evidence blobs.  
2. On `SignaturePackageCompleted` involving the person, People appends history ref.  
3. Voided packages update history status via Signatures events.

### 14.10 Notification Preferences

1. Preference SoR is Notifications (channel, quiet hours, etc.).  
2. People profile may show a summary fetched via Notifications query API.  
3. Do not duplicate preference tables in People.

### 14.11 History

1. Person History is an append/projection timeline combining People events + selected foreign events.  
2. History entries carry provenance refs; they are not a second audit SoR (Core Audit remains security audit).  
3. Medical detail bodies are not written into history projections—only coarse references.

---

## 15. How Other Modules Consume People

| Module | Consumption |
| --- | --- |
| **Core** | Links User↔Person; membership may include `PersonId`; deactivation events may trigger session/membership workflows |
| **Projects** | Roster names/trades via People queries; assignment orchestration checks active person |
| **Safety** | Participants identified by `PersonId`; may query FitForWork coarse signal; must not read medical notes without permission |
| **Equipment** | Operator `PersonId` on inspections; active person checks |
| **Training** | Targets requirements by person/trade/role; emits completions that People projects |
| **Documents** | Acknowledgements by person; cert document refs on profile cards |
| **Signatures** | Signer person refs; People stores history only |
| **COR** | Uses person/cert projections as evidence indexes with provenance back to Training/Documents/Signatures |
| **Notifications** | Resolves contact channels; owns preferences |
| **Workflows** | Onboarding: create person → invite user → assign membership |
| **Analytics** | Person dimensions from events (no PHI in warehouse by default) |
| **Web UX** | Directory & profile tabs: Overview, Training, Certs, Assignments, Availability, Attendance, Signatures, History |

### 15.1 Typical Read Path (Supervisor)

```text
Open Person Profile
  → People.GetPerson
  → Training status via Training API / People competency projection
  → Assignments via People view (Core-backed)
  → Medical banner via GetFitForWorkSignal (+ details if authorized)
  → Signature history refs
```

### 15.2 Typical Write Path (Restrict Worker)

```text
RecordMedicalRestriction
  → Core.Authorize(people.medical.manage)
  → People invariants
  → persist + FitForWorkSignalChanged (coarse)
  → Core.AuditApi.Append
  → Safety/Projects consumers refresh eligibility UX
```

---

## 16. Data Ownership

### 16.1 Schema `people` Owns

- Persons, roles, trades, contacts  
- Emergency contacts, medical restrictions  
- Employments, contractor engagements  
- Availability, attendance  
- Certification profile entries (refs)  
- Competency / assignment / signature-history / person-history projections  

### 16.2 Forbidden in People

- Password hashes, sessions, grants  
- Authoritative project membership rows  
- Training completion ledgers  
- Signature bitmap/evidence packages  
- Notification preference SoR tables  
- Safety activity records  

### 16.3 Privacy & Retention

- Health and emergency data: separate retention class; export controls  
- Events scrubbed of PHI where possible  
- Align retention with Core settings / legal hold policies  

---

## 17. Consistency & Workflows

| Flow | Pattern |
| --- | --- |
| Register + invite user | Temporal: People.RegisterPerson → Core.InviteUser → Link |
| Membership assign | Projects/Core command; People projection eventual |
| Training expiry | Training workflow; People competency projection updates on event |
| Medical restriction | Sync write in People; coarse signal eventual to consumers |
| Attendance sync (offline) | Idempotent People commands with mutation ids |

---

## 18. Anti-Patterns

1. Treating Workforce Role as AuthZ  
2. Copying Training completions into mutable People “truth” tables  
3. Capturing signatures inside People  
4. Using People attendance as COR-sealed toolbox proof  
5. Broadcasting medical notes on NATS  
6. SQL joins from Safety into `people` medical tables  
7. Creating a person-per-project duplicate profile  

---

## 19. Success Criteria

People is correctly designed when:

1. Every field actor has a stable `PersonId` used across Safety, Training, and Equipment.  
2. Users authenticate via Core; persons operate via People.  
3. Supervisors can see eligibility signals without People owning Training/Equipment rules.  
4. Medical/emergency data is tightly permissioned and minimally evented.  
5. Assignment, signature, and training UIs feel unified on the profile while ownership remains modular.  
6. History explains what happened to a person without becoming a shadow audit store.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Principal Architecture | Initial People domain design (`people`; replaces workforce naming) |

---

*End of People Domain Architecture*
