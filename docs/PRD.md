# Proven — Product Requirements Document (PRD)

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Product Requirements Document |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Product |
| **Audience** | Engineering, Design, Leadership, GTM |
| **Last updated** | 2026-08-03 |

---

## 1. Executive Summary

Proven is a **Construction Compliance Operating System** for contractors who must keep people safe, projects compliant, and audits defensible across complex, multi-party job sites.

It is not a digital clipboard or a standalone safety-forms app. Proven is the system of record and system of action for construction compliance: projects, people, safety activities, equipment, documents, signatures, training, COR audit readiness, workflows, analytics, and administration—operating as one cohesive platform.

Proven serves General Contractors, Prime Contractors, Subcontractors, Crane Companies, Concrete Forming Companies, Civil Contractors, and Industrial Contractors across **Canada, the United States, Australia, and New Zealand**.

The product is **mobile-first for workers** and **desktop-first for supervisors, safety coordinators, project managers, and administrators**, with offline-first field capability, strong auditability, and workflow-driven enforcement of how compliance work actually gets done.

---

## 2. Vision

To be the operating system that construction organizations trust to prove—every day, on every site—that their people, equipment, and work are compliant, capable, and audit-ready.

---

## 3. Mission

Give contractors a single, modular platform that:

1. Captures compliance activity where work happens (field and office).
2. Connects people, projects, equipment, documents, training, and safety into one operational picture.
3. Enforces consistent processes through workflows, permissions, and audit trails.
4. Turns compliance data into actionable insight for leadership and auditors.
5. Scales across companies, trades, regions, and regulatory frameworks without becoming a fragmented toolkit.

---

## 4. Core Philosophy

### 4.1 Construction Compliance Operating System

Proven treats compliance as an operating concern, not a paperwork afterthought.

- **One platform, many domains** — Projects, People, Safety, Equipment, Documents, Signatures, Training, COR Audit, Analytics, Notifications, Workflows, and Administration are integrated modules of one system.
- **Proof over forms** — Forms are inputs; the product outcome is defensible evidence of compliance.
- **Field reality first** — Workers and supervisors operate under time pressure, poor connectivity, and multi-contractor coordination. The product must work in those conditions.
- **Process as product** — Temporal workflows and event-driven module boundaries encode how work moves: assignments, escalations, approvals, closures, and audits.
- **Long-term maintainability** — Prefer modular ownership, clear interfaces, and durable architecture over short-term feature convenience.
- **Trust through auditability** — Every meaningful action is attributable, timestamped, and reviewable.

### 4.2 What Proven Is Not

- Not a generic form builder sold as “safety software.”
- Not a disconnected set of micro-apps (toolbox talks in one place, training in another, equipment elsewhere).
- Not a permanent-storage cache or spreadsheet replacement with a UI.
- Not a system that places business rules in the client or in background workers as a substitute for domain ownership.

---

## 5. Business Goals

| Goal | Description | Horizon |
| --- | --- | --- |
| **Establish system-of-record status** | Become the primary compliance system for target contractor accounts | 0–18 months |
| **Drive multi-module adoption** | Customers use ≥4 core modules (e.g., Projects, People, Safety, Documents) within 90 days of go-live | 0–12 months |
| **Reduce audit prep burden** | Cut COR / external audit preparation time through continuous evidence capture | 6–18 months |
| **Improve field completion** | Increase on-time completion of required safety and compliance activities | 0–12 months |
| **Expand regional readiness** | Support CA, US, AU, NZ operating models, language/locale needs, and common frameworks | 0–24 months |
| **Enable enterprise procurement** | Meet security, SSO, RBAC, retention, and admin requirements expected by mid-market and enterprise contractors | 0–18 months |
| **Create durable differentiation** | Compete as a Compliance OS (workflow + evidence + cross-domain integration), not on form count | Ongoing |

---

## 6. Target Market

### 6.1 Primary Segments

| Segment | Why they buy |
| --- | --- |
| **General / Prime Contractors** | Multi-trade coordination, site-wide compliance visibility, subcontractor accountability |
| **Specialty Subcontractors** | Trade-specific compliance, portable worker/equipment records, GC-imposed requirements |
| **Crane Companies** | Equipment readiness, operator competency, lift-related documentation and sign-offs |
| **Concrete Forming Companies** | Crew/site compliance, high-risk task controls, document and signature workflows |
| **Civil Contractors** | Distributed sites, equipment fleets, training currency, inspection evidence |
| **Industrial Contractors** | Permit-like rigor, contractor orientation, strict audit and access control needs |

### 6.2 Company Profile (Initial ICP)

- Mid-market to enterprise construction organizations
- Multi-project and/or multi-site operations
- Mix of office administrators and field workforce
- Existing pain with spreadsheets, paper, fragmented SaaS tools, or audit scramble
- Need to demonstrate compliance to clients, regulators, insurers, and COR auditors

### 6.3 Geographic Focus

| Priority | Regions |
| --- | --- |
| **Primary** | Canada, United States |
| **Near-term expansion** | Australia, New Zealand |
| **Design constraint** | Multi-region from day one in data model, permissions, document types, and workflow configuration—even if GTM is sequenced |

### 6.4 Buying Centers

- Safety / HSE leadership
- Compliance / Quality leadership
- Operations / Project leadership
- IT / Security (enterprise deals)
- Executive sponsors (risk, insurance, brand protection)

---

## 7. User Personas

### 7.1 Field Worker — “Alex”

- **Role:** Labourer, tradesperson, operator, apprentice
- **Goals:** Complete required tasks quickly; stay eligible to work; avoid paperwork friction
- **Needs:** Mobile/PWA experience, offline support, clear assigned tasks, simple signatures, training reminders
- **Pain:** Slow apps, unclear requirements, lost paper forms, repeated orientation at every site
- **Success:** “I know what’s required of me today, and I can finish it on my phone—even with bad signal.”

### 7.2 Supervisor / Foreman — “Sam”

- **Role:** Crew lead, foreman, site supervisor
- **Goals:** Keep crew productive and compliant; close daily requirements; escalate issues fast
- **Needs:** Crew roster visibility, incomplete-task queues, toolbox talks / FLHA workflows, equipment checks, notifications
- **Pain:** Chasing signatures, not knowing who is trained/competent, fragmented tools
- **Success:** “I can see my crew’s compliance status and clear blockers before work starts.”

### 7.3 Safety Coordinator — “Jordan”

- **Role:** Safety advisor / HSE coordinator
- **Goals:** Standardize practice across sites; investigate incidents; prepare for audits
- **Needs:** Configurable safety processes, document control, COR evidence packs, analytics, audit trails
- **Pain:** Inconsistent execution across projects; evidence scattered across email and drives
- **Success:** “I can prove what happened, who signed, and what was current—without a week of archaeology.”

### 7.4 Project Manager — “Priya”

- **Role:** Project / site manager
- **Goals:** Keep project on schedule and contractually compliant; manage subs
- **Needs:** Project-level compliance dashboards, subcontractor accountability, document packages, escalation workflows
- **Pain:** Late discoveries of non-compliance; weak sub visibility
- **Success:** “I know which contractors and crews are green, amber, or red before it becomes a stop-work event.”

### 7.5 Equipment / Yard Manager — “Chris”

- **Role:** Equipment manager, yard lead, crane operations support
- **Goals:** Keep assets inspectable, certified, and assigned correctly
- **Needs:** Equipment registry, inspections, certifications, assignment to projects/people, expiry alerts
- **Pain:** Expired certs discovered too late; unclear custody of assets
- **Success:** “No asset goes to site without current inspections and documents.”

### 7.6 Training Administrator — “Taylor”

- **Role:** Training / competency administrator
- **Goals:** Maintain workforce competency matrix; assign and track training
- **Needs:** Training records, expiries, required-vs-completed by role/project, certificates/documents
- **Pain:** Spreadsheet competency matrices; no link to site access or task eligibility
- **Success:** “Training status is live, enforced, and connected to who can do what.”

### 7.7 Company Administrator — “Morgan”

- **Role:** Platform / company admin
- **Goals:** Configure org structure, roles, integrations, retention, and access
- **Needs:** Tenant administration, RBAC, org units, project setup patterns, notification policies, audit logs
- **Pain:** Rigid systems that require vendors for every change
- **Success:** “I can onboard a company, projects, and roles without engineering involvement.”

### 7.8 Executive Sponsor — “Riley”

- **Role:** Director / VP Safety, COO, risk owner
- **Goals:** Reduce incident and audit risk; demonstrate control to clients and insurers
- **Needs:** Portfolio analytics, trend lines, exception reporting, confidence in data integrity
- **Pain:** Vanity dashboards disconnected from field reality
- **Success:** “I can trust the numbers because the operating system underneath is enforced.”

---

## 8. User Stories

Stories are written as outcomes. Acceptance detail lives in functional requirements and later epics.

### 8.1 Projects

- As a **project manager**, I want to create and configure a project so that all compliance activity is scoped correctly.
- As a **supervisor**, I want to see project-required activities for my crew so that daily work starts compliant.
- As a **safety coordinator**, I want project templates and required controls so that standards apply consistently across sites.
- As an **admin**, I want to manage project membership and contractor participation so that access matches site reality.

### 8.2 People

- As an **admin**, I want a people directory with roles, trades, and employment/contractor relationships so that identity is authoritative.
- As a **supervisor**, I want to know who is on my crew today and whether they are eligible to work so that I can prevent non-compliant deployment.
- As a **worker**, I want a single profile for certifications and acknowledgements so that I am not re-entering the same information.
- As a **GC**, I want visibility into subcontractor personnel status on my project so that I can enforce site requirements.

### 8.3 Safety

- As a **worker**, I want to complete assigned safety activities (e.g., FLHA, toolbox talk acknowledgement, hazard report) on mobile, including offline, so that I can comply without leaving the field.
- As a **supervisor**, I want to run and close crew safety activities with digital attendance/sign-off so that evidence is captured in real time.
- As a **safety coordinator**, I want configurable safety workflows and escalations so that serious issues are never silently dropped.
- As a **project manager**, I want project safety status and open actions so that risk is visible operationally—not only after an incident.

### 8.4 Equipment

- As an **equipment manager**, I want to register assets with types, ownership, and documents so that the fleet is governed.
- As a **supervisor**, I want to confirm equipment inspection/cert status before use so that unsafe or expired equipment is not deployed.
- As a **worker/operator**, I want guided pre-use checks with signature capture so that inspections are consistent and attributable.
- As a **safety coordinator**, I want expiry and failure alerts so that remediation happens before work is blocked or risk accumulates.

### 8.5 Documents

- As a **safety coordinator**, I want controlled document libraries (policies, SWPs, SDS, permits, site docs) with versioning so that teams use current controlled copies.
- As a **worker**, I want to view required documents for my project/task so that I can follow the latest instructions.
- As a **project manager**, I want project document packages and distribution tracking so that contractual and site document requirements are met.
- As an **auditor**, I want immutable version history and access logs so that document control is defensible.

### 8.6 Digital Signatures

- As a **worker**, I want to sign acknowledgements and forms digitally so that paper is not required.
- As a **supervisor**, I want multi-party sign-off flows so that crew and leadership approvals are complete.
- As a **compliance lead**, I want signature evidence bound to identity, timestamp, document version, and context so that signatures are audit-grade.
- As an **admin**, I want signature policies by document/process type so that wet-ink equivalents meet company standards.

### 8.7 Training

- As a **training admin**, I want to define required training by role, trade, project, or task so that competency rules are explicit.
- As a **worker**, I want to see due/expired training and complete or upload proof so that I remain eligible.
- As a **supervisor**, I want crew training gaps surfaced before assignment so that I do not put untrained people on restricted work.
- As a **safety coordinator**, I want training records linked to people and projects for COR and client audits.

### 8.8 COR Audit

- As a **safety coordinator**, I want continuous COR evidence collection mapped to audit elements so that audit prep is not a fire drill.
- As an **executive**, I want readiness scoring and gap lists so that investment and attention go to weak areas.
- As an **auditor (internal)**, I want exportable evidence packages with provenance so that submissions are complete and traceable.
- As an **admin**, I want configurable COR frameworks/versions relevant to region and certifying body so that the product fits local programs.

### 8.9 Analytics

- As an **executive**, I want portfolio dashboards for compliance health, incidents/near misses (as captured), training currency, and equipment readiness.
- As a **project manager**, I want project exception views so that I can intervene early.
- As a **safety coordinator**, I want trend and hotspot analysis so that programs improve, not only report.
- As an **ops analyst**, I want trustworthy metrics derived from audited operational events—not manual vanity inputs.

### 8.10 Notifications

- As a **user**, I want timely notifications for assignments, escalations, expiries, and approvals so that nothing critical is missed.
- As an **admin**, I want configurable notification rules and channels so that noise is controlled.
- As a **supervisor**, I want digest and priority modes so that I can manage attention during shift work.

### 8.11 Workflows

- As a **safety coordinator**, I want durable multi-step workflows (assign → complete → review → close → escalate) so that compliance processes are enforced consistently.
- As an **admin**, I want configurable workflow templates per company/project without breaking audit history.
- As an **engineer/operator of the platform**, I want workflows to be the source of business process truth (not ad-hoc client logic).

### 8.12 Administration

- As a **company admin**, I want org structure, roles/permissions, projects, contractors, and integrations managed centrally.
- As a **security stakeholder**, I want SSO, least-privilege RBAC, session controls, and full admin audit logs.
- As a **customer success / onboarding lead**, I want repeatable tenant setup so that time-to-value is short.

---

## 9. Functional Requirements

Requirements are organized by capability. Priority key: **P0** = MVP / launch-critical, **P1** = near-term, **P2** = later.

### 9.1 Projects

| ID | Requirement | Priority |
| --- | --- | --- |
| PRJ-01 | Create, update, archive projects with unique identity, status, location/region, and metadata | P0 |
| PRJ-02 | Assign companies, people, and roles to projects | P0 |
| PRJ-03 | Support prime/subcontractor participation models on a project | P0 |
| PRJ-04 | Define project-required compliance controls (activities, docs, training, equipment rules) | P0 |
| PRJ-05 | Project templates for repeatable setup | P1 |
| PRJ-06 | Project-level activity feeds and compliance status summary | P1 |
| PRJ-07 | Multi-site / multi-area structure within a project | P2 |

### 9.2 People

| ID | Requirement | Priority |
| --- | --- | --- |
| PPL-01 | People profiles (identity, contact, trade/role, employer/contractor linkage) | P0 |
| PPL-02 | Employment and contractor relationship modeling | P0 |
| PPL-03 | Role-based eligibility signals (training, documents, acknowledgements) | P0 |
| PPL-04 | Crew / team groupings for supervisors | P1 |
| PPL-05 | Visitor / temporary worker profiles with limited scope | P1 |
| PPL-06 | Cross-project people history for internal workforce | P1 |
| PPL-07 | Competency matrix views | P2 |

### 9.3 Safety

| ID | Requirement | Priority |
| --- | --- | --- |
| SAF-01 | Configurable safety activity types (e.g., FLHA/JSA, toolbox talk, inspection, incident/near miss, observation) | P0 |
| SAF-02 | Mobile completion UX with offline draft/sync | P0 |
| SAF-03 | Attendance, acknowledgements, and multi-party participation | P0 |
| SAF-04 | Corrective actions with owners, due dates, and closure | P0 |
| SAF-05 | Escalation rules for overdue or high-severity items | P0 |
| SAF-06 | Link safety records to project, people, equipment, and documents | P0 |
| SAF-07 | Incident investigation workflow and evidence attachments | P1 |
| SAF-08 | Stop-work / critical risk workflows | P1 |
| SAF-09 | Trade- or task-specific safety packs | P2 |

### 9.4 Equipment

| ID | Requirement | Priority |
| --- | --- | --- |
| EQP-01 | Equipment registry (identity, type, owner, status, project assignment) | P0 |
| EQP-02 | Inspection checklists and pre-use checks | P0 |
| EQP-03 | Certification/document attachments with expiry tracking | P0 |
| EQP-04 | Fail / out-of-service states that block assignment or use signals | P0 |
| EQP-05 | Notifications for upcoming/overdue inspections and cert expiries | P0 |
| EQP-06 | Custody / assignment history | P1 |
| EQP-07 | Crane and high-risk equipment specialized profiles | P1 |
| EQP-08 | Maintenance integration hooks | P2 |

### 9.5 Documents

| ID | Requirement | Priority |
| --- | --- | --- |
| DOC-01 | Document library with folders/collections and access control | P0 |
| DOC-02 | Versioning, effective dating, and controlled publish/retire | P0 |
| DOC-03 | Attach documents to projects, people, equipment, training, and safety records | P0 |
| DOC-04 | Require acknowledgement of specific document versions | P0 |
| DOC-05 | Object storage for binaries with secure access patterns | P0 |
| DOC-06 | Full-text search across document metadata and content (as available) | P1 |
| DOC-07 | Controlled distribution lists and read receipts | P1 |
| DOC-08 | Retention policies and legal hold | P2 |

### 9.6 Digital Signatures

| ID | Requirement | Priority |
| --- | --- | --- |
| SIG-01 | Capture signatures bound to user identity (or verified guest flow where permitted) | P0 |
| SIG-02 | Bind signature to record version, timestamp, project context, and device/session metadata | P0 |
| SIG-03 | Multi-signer workflows with sequence and completeness rules | P0 |
| SIG-04 | Immutable signature evidence package for audit export | P0 |
| SIG-05 | Signature policy configuration by process type | P1 |
| SIG-06 | Optional advanced trust options (e.g., stronger identity assurance) | P2 |

### 9.7 Training

| ID | Requirement | Priority |
| --- | --- | --- |
| TRN-01 | Training course/requirement catalog | P0 |
| TRN-02 | Assign requirements by role, trade, project, or individual | P0 |
| TRN-03 | Track completion, expiry, and evidence attachments | P0 |
| TRN-04 | Surface gaps to workers and supervisors | P0 |
| TRN-05 | Gate eligibility signals used by projects/safety/equipment processes | P1 |
| TRN-06 | Learning content hosting or links to external LMS | P2 |
| TRN-07 | Quizzes / knowledge checks | P2 |

### 9.8 COR Audit

| ID | Requirement | Priority |
| --- | --- | --- |
| COR-01 | Map operational evidence to COR (or equivalent) audit elements | P0 |
| COR-02 | Continuous readiness view with gaps and owners | P0 |
| COR-03 | Evidence package generation with provenance | P0 |
| COR-04 | Support region-specific COR/program variants via configuration | P1 |
| COR-05 | Internal audit scheduling and finding workflows | P1 |
| COR-06 | Benchmarking across projects/business units | P2 |

### 9.9 Analytics

| ID | Requirement | Priority |
| --- | --- | --- |
| ANA-01 | Operational dashboards: completion, overdue, training currency, equipment readiness | P0 |
| ANA-02 | Project and portfolio rollups with RBAC-aware visibility | P0 |
| ANA-03 | Export of standard reports | P0 |
| ANA-04 | Trend analysis and hotspot identification | P1 |
| ANA-05 | Executive scorecards | P1 |
| ANA-06 | Advanced analytical warehouse use cases (ClickHouse-backed) | P2 |

### 9.10 Notifications

| ID | Requirement | Priority |
| --- | --- | --- |
| NTF-01 | In-app notifications for assignments, approvals, escalations, expiries | P0 |
| NTF-02 | Email notifications for critical events | P0 |
| NTF-03 | User notification preferences within policy limits | P1 |
| NTF-04 | Push notifications for PWA/mobile where supported | P1 |
| NTF-05 | Digests and quiet hours | P2 |
| NTF-06 | SMS for critical escalations (opt-in / regulated use) | P2 |

### 9.11 Workflows

| ID | Requirement | Priority |
| --- | --- | --- |
| WFL-01 | Durable business workflows for core compliance processes (create → assign → complete → review → close) | P0 |
| WFL-02 | Escalations, timeouts, and reassignment | P0 |
| WFL-03 | Workflow templates configurable per tenant/project | P0 |
| WFL-04 | Event-driven triggers across modules (e.g., training expiry affects eligibility) | P0 |
| WFL-05 | Human-in-the-loop approvals | P0 |
| WFL-06 | Visible workflow status to end users (“where is this?”) | P1 |
| WFL-07 | Cross-company workflows for GC/sub interactions | P2 |

### 9.12 Administration

| ID | Requirement | Priority |
| --- | --- | --- |
| ADM-01 | Multi-tenant company administration | P0 |
| ADM-02 | RBAC with least-privilege roles and project-scoped permissions | P0 |
| ADM-03 | Org units / business units | P1 |
| ADM-04 | SSO (enterprise) | P1 |
| ADM-05 | Audit log of administrative and sensitive actions | P0 |
| ADM-06 | Data retention and export tooling | P1 |
| ADM-07 | Feature flags / module enablement per tenant | P1 |
| ADM-08 | API keys / integration administration | P2 |

### 9.13 Cross-Cutting Platform Requirements

| ID | Requirement | Priority |
| --- | --- | --- |
| PLT-01 | Modular monolith domain boundaries with public interfaces/events/workflows only | P0 |
| PLT-02 | API-first contracts for all client surfaces | P0 |
| PLT-03 | Offline-first field flows with conflict-safe sync for supported records | P0 |
| PLT-04 | Full audit trail for create/update/delete/sign/approve/close on compliance records | P0 |
| PLT-05 | Strong typing across API and domain boundaries | P0 |
| PLT-06 | Accessibility-compliant UI for worker and admin surfaces | P0 |
| PLT-07 | PWA installability for worker mobile use | P0 |

---

## 10. Non-Functional Requirements

### 10.1 Reliability & Workflow Integrity

- Business workflows must be durable and recoverable; process state must not live only in the client.
- Background workers may execute tasks but must not own business rules.
- Target availability for production cloud environments: **99.9%** monthly (exclusions for planned maintenance to be defined in SLA).

### 10.2 Performance

- Interactive page/API p95 for common read operations: **< 300 ms** server-side under normal load (exclusive of large file transfer).
- Mobile form open-to-interactive: **< 2 s** on mid-tier devices with warm cache.
- Offline queues must sync efficiently on reconnect without user data loss.

### 10.3 Scalability

- Support multi-tenant usage from single-site subcontractors to multi-project primes.
- Architecture must allow horizontal scale of API, workers, and workflow infrastructure.
- Analytics path must not degrade transactional operational performance (separate analytical store as product matures).

### 10.4 Security

- Security-first design: authentication, authorization, encryption in transit, encryption at rest for sensitive stores/objects.
- Tenant isolation enforced in every data access path.
- Secrets never stored in source control; least-privilege cloud roles.
- Admin and compliance actions fully auditable.
- Vulnerability management and dependency scanning in CI.

### 10.5 Privacy & Compliance

- Support data residency / regional deployment considerations for CA, US, AU, NZ over time.
- Configurable retention aligned to customer policy and legal requirements.
- Privacy-by-design for worker personal data; access strictly RBAC-scoped.

### 10.6 Offline & Mobile

- Core worker flows available offline with clear sync status and conflict handling.
- Mobile-first UX for workers; desktop-first for command-and-control roles.
- PWA install and resilient networking assumptions for construction sites.

### 10.7 Accessibility

- Meet **WCAG 2.2 AA** for primary workflows where technically feasible.
- Support large-tap targets, readable typography, and assistive technologies for field and office users.

### 10.8 Auditability & Data Integrity

- Immutable or append-only evidence patterns for signatures and critical approvals where required.
- Every compliance-significant action attributable to actor, time, and context.
- Redis is cache only—never system of record.
- PostgreSQL is the operational system of record; object storage for binaries; analytical store for heavy analytics when required.

### 10.9 Usability & Localization

- Language/locale readiness for primary markets; initial English with extensible i18n architecture.
- Terminology configurable where regional safety language differs (without forking the product).

### 10.10 Operability

- Structured logging, metrics, tracing.
- Safe migrations, backward-compatible APIs where practical.
- CI/CD via GitHub Actions; environment promotion discipline.

---

## 11. Success Metrics

### 11.1 Product Adoption

| Metric | Target (initial) |
| --- | --- |
| Time-to-first-project live | ≤ 14 days from tenant creation for standard onboarding |
| Modules adopted per active customer (90 days) | ≥ 4 core modules |
| Weekly active supervisors / licensed supervisors | ≥ 60% |
| Weekly active workers among invited field users | ≥ 40% (site- and season-adjusted) |

### 11.2 Operational Compliance Health

| Metric | Target (initial) |
| --- | --- |
| On-time completion rate for required safety activities | ≥ 90% for configured required activities |
| Overdue corrective actions older than policy threshold | Decreasing month-over-month after baseline |
| Training currency for required roles | ≥ 95% for active project-assigned workers |
| Equipment with valid inspection/cert at time of assignment | ≥ 98% |

### 11.3 Audit Readiness

| Metric | Target (initial) |
| --- | --- |
| COR evidence coverage for mapped elements | ≥ 85% continuous coverage before external audit cycle |
| Audit package generation time | < 1 day for standard package vs. multi-week manual assembly baseline |
| Audit finding recurrence on process/evidence gaps | Downward trend across cycles |

### 11.4 Platform Quality

| Metric | Target (initial) |
| --- | --- |
| Critical P0 incident rate | Near-zero in production; same-day response |
| Sync failure rate for offline submissions | < 1% unrecoverable; all recoverable failures visible to user |
| Customer-reported “source of truth” confidence (survey) | ≥ 4.5 / 5 after 6 months of use |

### 11.5 Business Outcomes

| Metric | Target (directional) |
| --- | --- |
| Net revenue retention | Expansion via seats, projects, and modules |
| Sales cycle win rate vs. form-centric competitors | Win on OS narrative + proof of integration depth |
| Support burden per tenant | Decline as admin self-serve and templates mature |

---

## 12. Future Roadmap

Roadmap is directional and will be re-planned per discovery and customer evidence.

### Phase 0 — Foundation (Platform Spine)

- Modular monolith boundaries, identity/RBAC, Projects, People
- Documents + Digital Signatures core
- Workflow engine integration for core processes
- Audit logging, notifications (in-app/email), admin basics
- Worker PWA shell with offline foundation

### Phase 1 — Compliance Operations MVP

- Safety activities + corrective actions
- Equipment registry + inspections/expiries
- Training requirements and currency
- Project compliance status
- COR evidence mapping (initial framework pack)
- Operational analytics (foundational dashboards)

### Phase 2 — Scale & Enterprise Readiness

- SSO, advanced admin, retention controls
- Stronger GC/sub multi-party workflows
- Deeper COR program variants (CA and expansion regions)
- Incident investigation depth
- Search upgrades as volume requires
- Push notifications and refined mobile UX

### Phase 3 — Intelligence & Ecosystem

- Advanced analytics / ClickHouse-backed explorations
- Hotspoting, predictive expiry/risk assists (human-supervised)
- Integrations (ERP/HRIS/LMS/equipment systems) via governed APIs
- OpenSearch (when PostgreSQL FTS is insufficient)
- Richer cross-tenant benchmark insights (privacy-preserving)

### Phase 4 — Regional Depth & Trade Packs

- Australia / New Zealand program depth
- Trade-specific packs (crane, forming, civil, industrial)
- Expanded controlled document intelligence and distribution
- Partner/channel implementation tooling

---

## 13. Out of Scope

The following are **out of scope for the initial Compliance OS PRD** (may be reconsidered later):

- Full HRIS / payroll replacement
- Full accounting / ERP replacement
- Generic no-code app builder for unrelated business domains
- Real-time GPS worker surveillance products as a core offering
- Medical diagnosis or clinical systems
- Being a general-purpose LMS content marketplace (integrations preferred)
- Social networking / non-compliance communication suites
- Heavy BIM/CAD authoring
- Autonomous decision-making that bypasses accountable human workflows
- Using Redis (or cache) as authoritative permanent storage
- Embedding core business rules in React clients or Go workers

---

## 14. Risks

| Risk | Impact | Likelihood | Mitigation |
| --- | --- | --- | --- |
| Perceived as “another forms app” | High | Medium | Lead with OS narrative, cross-module workflows, audit evidence, and outcome metrics in GTM and UX |
| Scope explosion across trades/regions | High | High | Modular boundaries; configurable frameworks; phased regional packs; strict P0 discipline |
| Field connectivity / offline complexity | High | High | Offline-first architecture for priority flows; clear sync UX; limit offline surface area initially |
| Multi-party GC/sub permission complexity | High | Medium | Explicit participation model; project-scoped RBAC; careful defaults |
| Audit/legal expectations for e-signatures vary by region | Medium | Medium | Signature evidence model + configurable policies; legal review per market |
| Customers demand customization that forks the product | High | Medium | Workflow/template configuration over code forks; module flags |
| Data migration from spreadsheets/legacy tools | Medium | High | Import tooling and professional services playbooks |
| Competing point solutions already entrenched | Medium | High | Land with painful wedge (e.g., COR readiness + field completion), expand modules |
| Overbuilding analytics before data quality exists | Medium | Medium | Instrument operational events first; analytics follows trustworthy capture |
| Regulatory change across CA/US/AU/NZ | Medium | Medium | Config-driven controls; avoid hardcoding jurisdiction logic into unrelated modules |

---

## 15. Assumptions

1. Construction customers will adopt a multi-module platform if it reduces audit pain and field friction—not if it only digitizes forms.
2. Workers will use a PWA/mobile web experience if it is fast, offline-capable, and task-focused.
3. Supervisors and safety teams will standardize on desktop for command-and-control while remaining mobile-capable.
4. COR (and regional equivalents) remain strategically important buying triggers in primary markets.
5. Temporal (or equivalent durable workflow technology) is the correct backbone for business processes; clients are not the process engine.
6. A modular monolith is the right initial architecture for speed with domain isolation; extract only when boundaries and scale demand it.
7. PostgreSQL can serve transactional needs and initial search; specialized search/analytics arrive when justified.
8. Customers accept that true compliance requires identity-bound actions and audit trails, including resistance to anonymous/shared logins.
9. Initial GTM can sequence geographies while the data model remains multi-region capable.
10. Success depends as much on onboarding, templates, and change management as on feature count.
11. Security and tenant isolation are non-negotiable prerequisites for enterprise procurement.
12. “Done” for a compliance feature includes evidence, workflow, permissions, audit, and reporting—not only UI capture.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Product | Initial PRD for Proven Construction Compliance Operating System |

---

*End of PRD*
