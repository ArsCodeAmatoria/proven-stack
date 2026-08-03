# Proven — Authorization & RBAC Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Authorization / RBAC Architecture |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | RBAC / Security Architecture |
| **Audience** | Security, Backend, Frontend, Product, Admin |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Core Domain](./CORE_DOMAIN.md), [Security Architecture](./SECURITY_ARCHITECTURE.md), [Authentication](./AUTHENTICATION_ARCHITECTURE.md), module domain docs, [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document designs Proven’s **authorization** model: companies and projects as scopes, roles and permissions, claims, feature entitlements, document/equipment/training/COR permission sets, administration, **delegation**, and **temporary permissions**.

**Hard rules**

1. **Core `AuthzApi` is the only permission authority** — modules never invent parallel AuthZ systems.  
2. **UI hiding is non-authoritative** — every command/query re-checks server-side.  
3. **Fail closed** — missing grant, ambiguous scope, disabled module, or expired temporary grant → deny.  
4. **JWT does not carry permission lists** — claims are identity/assurance only; AuthZ is evaluated live.  
5. **Sealed evidence immutability** is enforced by domain state + permissions (e.g. no silent edit).

**Documentation only — no implementation.**

---

## 2. Mental Model

```text
Allow iff
  authenticated principal
  ∧ tenant active
  ∧ license / feature entitlement allows module capability
  ∧ ∃ grant (role or temporary) whose scope covers the resource
  ∧ role (or temp set) contains PermissionCode
  ∧ ABAC constraints pass (resource attrs, assurance, classification)
  ∧ resource state allows action (e.g. not sealed for edit)
```

| Concept | Meaning |
| --- | --- |
| **Principal** | User or service identity |
| **Permission** | Atomic capability code (`documents.version.publish`) |
| **Role** | Named bundle of permissions |
| **Grant** | Role bound to principal **within a Scope** |
| **Scope** | Tenant · OrgUnit · Company · Project · Team · Self |
| **Claim** | AuthN token assertion (`sub`, `tid`, `sid`, `amr`) — **not** AuthZ grants |
| **Entitlement** | License/feature flag gate (module on/off, seat limits) |
| **Delegation** | Time-bounded grant of authority from one principal to another |
| **Temporary permission** | Explicit short-lived permission or role grant with expiry |

---

## 3. Companies

### 3.1 Role in AuthZ

| Concept | AuthZ meaning |
| --- | --- |
| **Company** | Legal/operating entity in a tenant (owner GC, sub, crane co., …) |
| **Company scope** | Grants limited to resources tagged with that company (participants, employments, some docs) |
| **Not a login tenant** | Tenant isolates customers; companies nest inside a tenant |

### 3.2 Company-related permissions (Core)

| Code | Meaning |
| --- | --- |
| `core.company.read` | View company profile |
| `core.company.manage` | Create/update companies |
| `core.company.archive` | Archive company |

### 3.3 Project participation

Company ↔ Project **participants** (Projects module) constrain which companies appear on a Place. Visibility of another company’s workers/docs uses **project scope + document ACL + membership**—not “same tenant ⇒ see all.”

GC/Sub separation: least privilege on shared projects; no shared tenant-wide admin by default.

---

## 4. Projects

### 4.1 Project as primary field scope

Most operational permissions are evaluated with **`Scope=Project(project_id)`** (or Team under project).

| Binding | Owner | AuthZ effect |
| --- | --- | --- |
| **Project Membership** | Core | Principal/person may act on that project with membership roles |
| **AccessGrant (Project)** | Core | Explicit role grant at project scope |
| **Participant (Company)** | Projects | Company is on the job—feeds filters, not a substitute for user grants |

### 4.2 Project permissions

| Code | Meaning |
| --- | --- |
| `projects.project.read` | View project / Place |
| `projects.project.create` | Create project (tenant/org scoped) |
| `projects.project.manage` | Update settings, areas, templates application |
| `projects.project.activate` | Activate lifecycle |
| `projects.project.archive` | Archive |
| `projects.participant.manage` | Add/remove company participants |
| `projects.membership.manage` | Request/coordinate membership (actual grant via Core) |

Membership grant/revoke: `core.membership.manage` (or equivalent Core codes) with project scope.

### 4.3 Scope coverage rules

| Grant scope | Covers |
| --- | --- |
| Tenant | All projects in tenant (admin-style)—use sparingly |
| OrgUnit | Projects under org (if org-linked) |
| Project | That project only |
| Team | Resources assigned to team within project |
| Self | Own person-bound records only |

List endpoints filter by allowed project ids—never trust client `project_id` alone.

---

## 5. Roles

### 5.1 Role kinds

| Kind | Description |
| --- | --- |
| **System roles** | Platform-defined; shipped with permission catalog; not arbitrarily deleted |
| **Tenant roles** | Custom bundles within policy (cannot exceed grantor’s assignable set) |
| **Membership roles** | Labels on ProjectMembership (Worker, Supervisor, …) that map to permission bundles or implied grants |

### 5.2 Representative system roles

| Role | Typical scope | Intent |
| --- | --- | --- |
| `tenant_admin` | Tenant | Full tenant configuration |
| `company_admin` | Company / Org | Company administration |
| `project_admin` / `pm` | Project | Place configuration & oversight |
| `safety_coordinator` | Project / Tenant | Safety cases, CA, incidents |
| `supervisor` | Project / Team | Crew reviews, assignments |
| `worker` | Project / Self | Field My Actions |
| `equipment_manager` | Project / Tenant | Assets, readiness, maintenance |
| `training_admin` | Tenant / Project | Courses, assignments |
| `document_control` | Tenant / Project | Controlled documents |
| `cor_admin` | Tenant | COR engagements & packages |
| `auditor_readonly` | Tenant / Project | Evidence read |
| `admin_operator` | Tenant | Administration console capabilities |
| `integration` | Tenant | API client / service principal |

### 5.3 Role administration permissions

| Code | Meaning |
| --- | --- |
| `core.role.read` | View roles |
| `core.role.manage` | Define/change tenant roles |
| `core.grant.manage` | Grant/revoke access |
| `core.grant.read` | List grants |

**Separation of duties:** granting `tenant_admin` may require dual control / step-up MFA (policy).

---

## 6. Permissions

### 6.1 Naming

```text
{module}.{resource}.{action}
```

Examples: `safety.activity.submit`, `equipment.asset.read`, `cor.package.generate`.

### 6.2 Catalog ownership

- Modules **propose** codes; Core **publishes** the catalog.  
- Breaking renames avoided; deprecate + add.  
- Each code has description, sensitivity class, and default roles.

### 6.3 Evaluation API

```text
AuthzApi.Authorize(principal, permission_code, resource_scope) → Allow | Deny(reasons)
AuthzApi.ListEffectivePermissions(principal, scope?) → set (UI hints)
AuthzApi.ListAccessibleProjectIds(principal) → ids for list filters
```

---

## 7. Claims (vs Permissions)

Claims live on **AuthN tokens** ([Authentication Architecture](./AUTHENTICATION_ARCHITECTURE.md)):

| Claim | AuthZ use |
| --- | --- |
| `sub` | Who |
| `tid` | Tenant isolation |
| `sid` | Session must be active |
| `amr` / `acr` | Step-up / MFA for sensitive permissions |
| `did` | Device binding anomalies |

**Claims are not grants.** A claim never replaces `Authorize(...)`.  
Optional future: `ent` entitlement snapshot for UX only—server still checks license.

---

## 8. Feature Permissions (Entitlements + Flags)

Two layers gate features before/with RBAC:

| Layer | Owner | Effect |
| --- | --- | --- |
| **License entitlement** | Core License | Module enabled; seat limits |
| **Feature flag** | Core Flags | Gradual rollout / tenant toggle |
| **Permission** | Core AuthZ | Who may use the enabled feature |

```text
Deny with module_disabled if license/flag off
else evaluate RBAC permission
```

| Examples | Meaning |
| --- | --- |
| COR module off | All `cor.*` deny regardless of role |
| Offline FLHA flag off | UX hides; API rejects offline submit type |
| Analytics export | Needs entitlement + `analytics.export.create` + often step-up |

Feature permissions are **not** a second RBAC— they are **preconditions** documented as entitlement checks in Authz composition.

---

## 9. Document Permissions

| Code | Meaning |
| --- | --- |
| `documents.document.read` | View metadata / effective docs in scope |
| `documents.document.manage` | Create/edit document records |
| `documents.version.create` | New draft version |
| `documents.version.publish` | Publish (often dual-control) |
| `documents.version.withdraw` | Withdraw |
| `documents.ack.manage` | Run acknowledgement campaigns |
| `documents.ack.complete` | Complete own ack |
| `documents.acl.manage` | Restrict audience / ACL |

### 9.1 ABAC overlays

| Attribute | Effect |
| --- | --- |
| Classification / restrict flag | Extra ACL beyond project membership |
| Version state | Draft vs published vs withdrawn |
| Effective dating | Read effective vs historical |

SWP/SJP use same document codes with type facets—not separate AuthZ systems.

---

## 10. Equipment Permissions

| Code | Meaning |
| --- | --- |
| `equipment.asset.read` | View assets / readiness display |
| `equipment.asset.manage` | Create/update assets |
| `equipment.inspection.perform` | Complete pre-use/periodic |
| `equipment.inspection.manage` | Configure inspection requirements |
| `equipment.cert.manage` | Manage certifications |
| `equipment.deficiency.manage` | Open/clear deficiencies |
| `equipment.maintenance.manage` | Maintenance orders |
| `equipment.readiness.override` | Restricted break-glass (audited) |
| `equipment.binder.manage` | Tower/self-erect binders |
| `equipment.oos.manage` | Take OOS / release |

Readiness **enforcement** is domain logic; permissions gate who may change inputs/overrides.

---

## 11. Training Permissions

| Code | Meaning |
| --- | --- |
| `training.course.read` | View courses |
| `training.course.manage` | Manage catalog |
| `training.requirement.manage` | Project/tenant requirements |
| `training.assignment.manage` | Assign training |
| `training.assignment.complete_self` | Complete own assignment |
| `training.completion.record` | Record completion (trainer) |
| `training.completion.revoke` | Revoke bad completion (audited) |
| `training.gap.read` | View competency gaps |
| `training.gap.manage` | Close/waive gaps per policy |

Currency **gates** (may worker perform X?) are Training domain queries used by other modules—AuthZ still required to invoke them.

---

## 12. COR Permissions

| Code | Meaning |
| --- | --- |
| `cor.framework.read` | View frameworks/elements |
| `cor.mapping.manage` | Map evidence to elements |
| `cor.readiness.read` | View readiness scores |
| `cor.gap.manage` | Own/close gaps |
| `cor.package.generate` | Generate evidence packages |
| `cor.package.read` | Download/view packages |
| `cor.engagement.manage` | Run internal/external prep engagements |
| `cor.engagement.close` | Close engagement / finalize score |
| `cor.admin` | Pack install / tenant COR settings |

Package generation often requires elevated assurance (MFA step-up).

---

## 13. Additional Module Permission Sets (Brief)

### 13.1 Safety

`safety.activity.create|submit|review|close|void`, `safety.ca.manage`, `safety.incident.manage`, `safety.permit.manage`, `safety.bulletin.manage`, …

### 13.2 Signatures

`signatures.package.create`, `signatures.package.void`, `signatures.slot.seal_self`, guest paths use **token AuthN** not RBAC roles.

### 13.3 People

`people.person.read|manage`, `people.attendance.manage`, …

### 13.4 Notifications / Analytics / Admin

See domain docs; Admin detailed in §14.

---

## 14. Administration

Administration is a **facade** ([Administration Domain](./ADMINISTRATION_DOMAIN.md)) with its own permissions **plus** underlying Core/module permissions for destructive actions.

| Code | Meaning |
| --- | --- |
| `admin.console.access` | Enter Administration |
| `admin.dashboard.read` | Admin home |
| `admin.branding.manage` | Branding |
| `admin.apikey.manage` | API keys |
| `admin.integration.manage` | Integrations |
| `admin.builder.edit` / `publish` | Builder drafts |
| `admin.health.read` | Tenant health |
| `admin.audit.view` | Audit viewer (also `core.audit.read`) |
| `core.flags.manage` | Feature flags |
| `core.license.read` | License view |

**Rule:** `admin.builder.publish` does not alone authorize publishing a Safety type into production without corresponding `safety.*` manage permission (defense in depth).

Privileged admin actions require **step-up MFA** (`acr`).

---

## 15. Delegation

### 15.1 Definition

**Delegation** = Principal A grants Principal B authority to act with a **subset** of A’s permissions within a scope for a **limited time** (and optional reason).

Use cases: PM on leave delegates project admin; safety lead delegates CA verification; vacation coverage.

### 15.2 Model

| Field | Meaning |
| --- | --- |
| `delegator_id` | Must hold permissions being delegated |
| `delegate_id` | Receiver |
| `scope` | Usually Project or Tenant slice |
| `permission_set` or `role_id` | Subset ≤ delegator effective set |
| `starts_at` / `ends_at` | Validity window |
| `reason` | Audited |
| `revocable` | Delegator, tenant_admin, or auto on expiry |

### 15.3 Rules

1. Cannot delegate permissions you do not hold.  
2. Cannot escalate (no granting `tenant_admin` via delegation unless policy explicitly allows and is dual-controlled).  
3. Nested delegation: default **forbidden** (delegate cannot re-delegate).  
4. All actions under delegation audit `acted_as_delegate_of`.  
5. UI shows “Acting for …” banner.  
6. Stored as Core grants with `grant_kind=delegation` + expiry.

### 15.4 Permissions to manage delegation

| Code | Meaning |
| --- | --- |
| `core.delegation.create` | Create delegation (within own rights) |
| `core.delegation.revoke` | Revoke |
| `core.delegation.read` | View |

---

## 16. Temporary Permissions

### 16.1 Forms

| Form | Description |
| --- | --- |
| **Timed role grant** | Standard grant with `expires_at` |
| **Timed permission grant** | Single permission emergency access |
| **Delegation** | §15 |
| **Break-glass** | Elevated temporary role with mandatory incident ticket / dual control |

### 16.2 Lifecycle

```text
Request (optional approval) → Grant with expires_at
  → AuthzApi honors only while now ∈ [start, end] ∧ not revoked
  → Expiry job/workflow revokes → audit TemporaryGrantExpired
```

### 16.3 Rules

1. Default deny after expiry without relying on client clocks (server time).  
2. Temporary grants appear in session/settings “active elevated access.”  
3. Break-glass notifies security channel; shorter max TTL.  
4. Cannot be refreshed silently—explicit renewal with audit.  
5. Combine with step-up MFA for privileged temp grants.

### 16.4 Workflow

`WorkflowAssignment` / dedicated Temporal timers may notify before expiry; Core remains SoR for grant validity.

---

## 17. ABAC Complements (Summary)

| Dimension | Examples |
| --- | --- |
| Resource | `document.restricted`, `incident.severity`, `asset.readiness` |
| Subject | `acr` MFA, active membership, person link |
| Environment | IP allowlist (optional), time window for temp grants |
| State | Sealed → deny mutate; voided → deny seal |

RBAC answers “which permission?”; ABAC answers “under what conditions?”

---

## 18. Service Principals & API Keys

| Principal | AuthZ |
| --- | --- |
| **API key / integration** | Role `integration` (or custom) at tenant scope with narrow permissions |
| **Go workers** | Service credentials; constrained callbacks; no user impersonation by default |
| **Impersonation** | Forbidden unless audited break-glass API |

---

## 19. Frontend Application

| Concern | Behavior |
| --- | --- |
| Capability flags | From `/auth/me` or effective-permissions endpoint |
| Hide/disable | UX only |
| Project switcher | Only accessible projects |
| Delegation banner | When acting under delegation |
| Temp access badge | When elevated grant active |

---

## 20. Audit Requirements

Must audit: grant/revoke, role definition changes, delegation create/revoke, temporary/break-glass grants, privilege use on sensitive resources (export, publish, void seal, COR finalize), admin API key lifecycle.

---

## 21. Testing Matrix (Representative)

| Case | Expect |
| --- | --- |
| Worker lacks `documents.version.publish` | 403 |
| Project A member cannot read Project B | Omit/403 |
| Restricted document | Project member still denied without ACL |
| Module license off | `module_disabled` |
| Expired temp grant | Deny |
| Delegate exceeds delegator rights | Create delegation rejected |
| JWT with forged role claim | Ignored; AuthZ deny without grant |

---

## 22. Success Criteria

1. Companies and projects have clear scope semantics; GC/Sub isolation holds.  
2. Roles and permission catalog are Core-published and module-complete (docs, equipment, training, COR, admin, …).  
3. Claims never substitute for Authorize.  
4. Feature entitlements compose with RBAC cleanly.  
5. Delegation and temporary permissions are time-bounded, least-privilege, and audited.  
6. Administration cannot bypass module manage permissions for dangerous publishes.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | RBAC Architecture | Full authorization design |

---

*End of Authorization & RBAC Architecture*
