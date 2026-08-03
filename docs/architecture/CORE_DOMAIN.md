# Proven — Core Domain Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Core Domain Architecture (DDD) |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Principal Architecture |
| **Audience** | Engineering, Security, Product |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Domain Model](./DOMAIN_MODEL.md), [System Architecture](./SYSTEM_ARCHITECTURE.md), [PRD](../PRD.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document defines the **Core** bounded context for Proven.

Core is the **platform foundation** of the Construction Compliance Operating System. Every other module depends on Core for tenancy, identity, access control, organizational structure, membership, shared file primitives, auditability, settings, feature flags, and licensing.

**This document contains no application code.**

### 1.1 Naming Clarification (DDD)

In Proven product language, **Core** means *platform foundation module*.

In strategic DDD classification, Core is a **generic / platform subdomain** (essential commodity capability), not the differentiating “core domain.” Differentiating domains remain Safety, Training, Equipment Compliance, Signatures, and COR Audit ([Domain Model](./DOMAIN_MODEL.md)).

| Term | Meaning here |
| --- | --- |
| **Core module** | Foundation bounded context (`core`) |
| **Core domain (DDD strategic)** | Differentiating compliance capabilities (other modules) |

---

## 2. Strategic Role

### 2.1 Why Core Exists

Without Core, every module would reinvent:

- Who the customer is (tenant)
- Who the actor is (user / principal)
- What they may do (roles, permissions, scopes)
- Where work is scoped (company, org, project membership, teams)
- How files are stored safely
- How actions are audited
- What the tenant is allowed to use (license + flags)

Core centralizes these concerns so compliance modules stay focused on proof and operations.

### 2.2 Consolidation Relative to Prior Map

Core **absorbs and supersedes** as a single module boundary:

| Prior context (v1 map) | Now in Core |
| --- | --- |
| Tenancy & Organization | Tenant, Company, Organization structure |
| Identity & Access | Users, authn, authz, roles, permissions, sessions |
| Platform Audit | Audit log streams |
| (partial Projects) | Project Membership (access binding only) |
| (new) | Teams, File Storage primitives, Settings, Feature Flags, Licensing |

**Not absorbed by Core:**

| Concern | Owning module |
| --- | --- |
| Project lifecycle, required controls, project templates | `projects` |
| Person HR/trade profile, crew-as-workforce concept | `workforce` |
| Controlled documents, versioning, acknowledgements | `documents` |
| Signature evidence packages | `signatures` |
| Safety / equipment / training / COR logic | respective modules |

> **Project Membership** lives in Core because it is an **authorization and participation binding**. **Project** as a construction place lives in `projects`.

---

## 3. Bounded Context Definition

### 3.1 Context Card

| Field | Value |
| --- | --- |
| **Name** | Core (Platform Foundation) |
| **Module** | `core` |
| **Type** | Generic / platform subdomain |
| **System of record for** | Tenants, companies, orgs, users, auth, roles, permissions, grants, project memberships, teams, file objects, audit entries, settings, feature flags, licenses |
| **Primary actors** | Platform admins, company admins, security, all authenticated principals (indirectly) |
| **Downstream** | Every Proven module |

### 3.2 Ubiquitous Language

| Term | Meaning |
| --- | --- |
| **Tenant** | Isolated customer workspace on the platform |
| **Company** | Legal/operating company known to a tenant (owner company or partner/sub) |
| **Organization / OrgUnit** | Hierarchical administrative structure within a tenant |
| **User** | Human account identity that can authenticate |
| **Principal** | Authenticated security subject (usually a User; may include service principals later) |
| **PersonRef** | Stable reference to a workforce person identity (owned by Workforce; referenced by Core) |
| **Role** | Named bundle of permissions |
| **Permission** | Atomic authorization capability (`code`) |
| **Grant** | Assignment of a role to a principal within a scope |
| **Scope** | Boundary for a grant: Tenant, OrgUnit, Project, Team, or Self |
| **Project Membership** | Binding of a principal/person to a project with membership role(s) |
| **Team** | Named group of people for operational assignment (tenant- or project-scoped) |
| **File Object** | Stored binary with metadata and access policy hooks (not a controlled document) |
| **Audit Entry** | Append-only record of a significant action |
| **Setting** | Tenant/org/user configuration value within a defined schema |
| **Feature Flag** | Runtime capability toggle (global/tenant/actor) |
| **License** | Commercial entitlement governing modules, seats, and limits |

### 3.3 Context Map Position

```text
                    ┌──────────────────────────────────────┐
                    │                 CORE                 │
                    │  Tenancy · Identity · Access · Org   │
                    │  Membership · Teams · Files · Audit  │
                    │  Settings · Flags · Licensing        │
                    └───────────────┬──────────────────────┘
                                    │
          public interfaces · events · authz decisions
                                    │
        ┌───────────┬───────────┬───┴────┬───────────┬───────────┐
        ▼           ▼           ▼        ▼           ▼           ▼
   Projects    Workforce    Safety   Equipment  Documents   Signatures
        │           │           │        │           │           │
        └───────────┴───────────┴────┬───┴───────────┴───────────┘
                                     ▼
                     Training · COR · Notifications · Workflows · Analytics
```

**Relationship pattern:** Core is an **Open Host Service** for identity, tenancy, authz, audit, files, settings, flags, and licensing. Downstream modules are **Conformist** to Core’s published IDs, permission codes, and event contracts.

---

## 4. Module Boundaries

### 4.1 Core Owns

1. Multi-tenant isolation model  
2. Companies and organizational hierarchy  
3. User accounts, credentials/SSO links, sessions  
4. Role & permission catalog and grants  
5. Authorization decision API  
6. Project membership bindings (not project definition)  
7. Teams and team membership  
8. File object storage intents and metadata (R2-backed)  
9. Platform audit log  
10. Settings registry and values  
11. Feature flags  
12. Licensing and seat/module entitlements  

### 4.2 Core Does Not Own

| Exclusion | Reason |
| --- | --- |
| Project create/close/templates/required controls | `projects` domain |
| Trade competency, employment HR attributes | `workforce` domain |
| Controlled document publish/ack semantics | `documents` domain |
| Signature evidence sealing rules | `signatures` domain |
| Safety/equipment/training/COR invariants | respective domains |
| Notification delivery providers | `notifications` + workers |
| Temporal orchestration definitions of compliance processes | `workflows` + domain modules |
| Analytical warehouses | `analytics` |

### 4.3 Hard Boundary Rules

1. No other module may read/write Core tables directly.  
2. No other module may implement its own parallel permission system.  
3. No other module may store authoritative tenant license state.  
4. File **bytes** live in object storage; Core owns **object identity + access metadata**. Controlled-document meaning stays in `documents`.  
5. Audit append is mandatory for Core-significant and compliance-significant actions (callers use Core Audit API).  
6. Redis may cache authz decisions; Core Postgres remains authority.  
7. Business rules for Safety/Training/etc. never move into Core “for convenience.”

### 4.4 Schema Ownership

- Schema: `core` (PostgreSQL)  
- Object prefixes: `r2://…/{tenant_id}/core/files/…`  
- Cache keys: `core:authz:…`, `core:flags:…` (TTL-bound, disposable)

---

## 5. Aggregate Roots

| Aggregate | Consistency boundary responsibility |
| --- | --- |
| **Tenant** | Workspace lifecycle, region defaults, status, isolation root |
| **Company** | Company profile within/related to tenant |
| **OrgUnit** | Hierarchical unit tree node and moves |
| **User** | Account identity, status, person link, credential/SSO bindings |
| **Session** | Authenticatable session lifecycle and revocation |
| **RoleDefinition** | Role name + permission set (system and tenant-custom where allowed) |
| **AccessGrant** | Principal ↔ Role ↔ Scope binding |
| **ProjectMembership** | Principal/Person ↔ Project participation and membership roles |
| **Team** | Team definition and membership |
| **FileObject** | Upload lifecycle, checksum, retention class, access class |
| **AuditStream** | Append-only audit partition for a tenant (logical aggregate) |
| **SettingsBundle** | Scoped settings document for tenant/org/user |
| **FeatureFlag** | Flag definition + targeting rules |
| **License** | Commercial entitlements, seats, module enablement, expiry |

> Prefer **smaller aggregates** and eventual consistency via events over a mega-`Tenant` aggregate that includes users, files, and grants.

---

## 6. Entities

### 6.1 Inside Tenancy / Org

| Entity | Parent aggregate | Notes |
| --- | --- | --- |
| `TenantStatusHistory` | Tenant | Optional history trail |
| `CompanyAffiliation` | Company / Tenant | How company relates to tenant (owner, partner) |
| `OrgUnitAssignment` | OrgUnit / User | User placement in org tree |

### 6.2 Inside Identity / Access

| Entity | Parent aggregate | Notes |
| --- | --- | --- |
| `Credential` | User | Password hash / webauthn — never logged |
| `ExternalIdentityLink` | User | SSO subject mapping |
| `PermissionBinding` | RoleDefinition | Permission codes on a role |
| `ScopeBinding` | AccessGrant | Concrete scope target IDs |
| `MfaFactor` | User | If/when MFA is enabled |

### 6.3 Inside Membership / Teams

| Entity | Parent aggregate | Notes |
| --- | --- | --- |
| `MembershipRoleBinding` | ProjectMembership | e.g., Worker, Supervisor on that project |
| `TeamMember` | Team | Person/User membership |

### 6.4 Inside Files / Audit / Config

| Entity | Parent aggregate | Notes |
| --- | --- | --- |
| `FileUploadIntent` | FileObject | Presign lifecycle |
| `AuditEntry` | AuditStream | Immutable row |
| `SettingEntry` | SettingsBundle | Key/value within schema |
| `FlagOverride` | FeatureFlag | Tenant/actor override |
| `SeatAllocation` | License | Consumed seats by type |
| `ModuleEntitlement` | License | Enabled modules |

---

## 7. Value Objects

### 7.1 Identifiers

- `TenantId`, `CompanyId`, `OrgUnitId`
- `UserId`, `PrincipalId`, `SessionId`
- `RoleId`, `PermissionCode`
- `ProjectId` *(reference only; authority in Projects)*
- `PersonId` *(reference only; authority in Workforce)*
- `TeamId`, `FileObjectId`, `AuditEntryId`
- `LicenseId`, `FeatureFlagKey`, `SettingKey`
- `CorrelationId`, `CausationId`

### 7.2 Enumerations / States

- `TenantStatus` — Active, Suspended, Closed  
- `UserStatus` — Invited, Active, Locked, Deactivated  
- `CompanyType` — Prime, Subcontractor, Crane, Forming, Civil, Industrial, Other  
- `GrantScopeType` — Tenant, OrgUnit, Project, Team, Self  
- `MembershipStatus` — Invited, Active, Suspended, Removed  
- `FileObjectStatus` — PendingUpload, Available, Quarantined, Deleted  
- `LicenseStatus` — Trial, Active, Grace, Expired, Suspended  
- `RegionCode` — CA, US, AU, NZ, …  

### 7.3 Structured Values

- `EmailAddress`, `PersonName`, `DisplayName`, `LegalName`
- `SecurePasswordHash` (write-only semantics)
- `OidcSubject`, `AuthProviderRef`
- `PermissionSet`
- `AccessScope` — `{ type, id? }`
- `IpAddress`, `UserAgent` (audit metadata)
- `ObjectStorageRef` — bucket/key/version
- `Checksum`, `ContentType`, `ByteSize`
- `RetentionClass`
- `SettingValue` (typed JSON per schema)
- `FlagTargetingRule`
- `SeatLimit`, `SeatType`
- `EffectivePeriod` / `Instant`

### 7.4 Shared Kernel Exports

Only these leave Core as widely shared primitives:

`TenantId`, `CompanyId`, `OrgUnitId`, `UserId`, `PrincipalId`, `PersonId` (ref), `ProjectId` (ref), `TeamId`, `FileObjectId`, `PermissionCode`, `RegionCode`, `CorrelationId`, `Instant`

No shared mutable aggregates.

---

## 8. Domain Services

Domain services host operations that don’t naturally fit a single aggregate but still express Core invariants.

| Domain service | Responsibility |
| --- | --- |
| **AuthenticationService** | Credential/SSO verification orchestration; emits session establishment (does not own external IdP) |
| **AuthorizationService** | Evaluate `Principal + PermissionCode + ResourceScope` → Allow/Deny with reason codes |
| **MembershipPolicyService** | Enforce uniqueness/conflicts for project membership and grant overlap rules |
| **LicenseEnforcementService** | Check module enabled, seat available, tenant license active before privileged provisioning |
| **FeatureFlagService** | Resolve effective flag for tenant/actor/environment |
| **SettingsResolver** | Resolve setting with precedence: User → OrgUnit → Tenant → Platform default |
| **FileAccessService** | Decide whether principal may upload/download a file object given grants + owner module policy hooks |
| **AuditRecorder** | Validate and append audit entries with integrity digests |
| **TenantProvisioningService** | Coordinate initial tenant + owner company + admin user + default roles + license bootstrap (may start a Temporal workflow for multi-step setup) |

> Application-layer services may wrap these for transactions/outbox. Domain services encode **rules**, not HTTP or SQL.

---

## 9. Domain Events

### 9.1 Tenancy & Organization

- `TenantProvisioned`
- `TenantSuspended`
- `TenantReactivated`
- `TenantClosed`
- `CompanyRegistered`
- `CompanyUpdated`
- `CompanyDeactivated`
- `OrgUnitCreated`
- `OrgUnitMoved`
- `OrgUnitArchived`

### 9.2 Users & Authentication

- `UserInvited`
- `UserActivated`
- `UserDeactivated`
- `UserLocked`
- `UserUnlocked`
- `UserProfileUpdated`
- `UserLinkedToPerson`
- `ExternalIdentityLinked`
- `SessionEstablished`
- `SessionRevoked`
- `AuthenticationFailed` (rate-limited / carefully audited)

### 9.3 Roles, Permissions, Grants

- `RoleDefinitionCreated`
- `RoleDefinitionChanged`
- `RoleDefinitionRetired`
- `AccessGranted`
- `AccessRevoked`
- `PermissionCatalogUpdated` (platform-level, rare)

### 9.4 Project Membership & Teams

- `ProjectMembershipGranted`
- `ProjectMembershipUpdated`
- `ProjectMembershipRevoked`
- `TeamCreated`
- `TeamUpdated`
- `TeamMemberAdded`
- `TeamMemberRemoved`
- `TeamArchived`

### 9.5 Files

- `FileUploadIntentCreated`
- `FileObjectAvailable`
- `FileObjectQuarantined`
- `FileObjectDeleted`
- `FileObjectRetentionChanged`

### 9.6 Audit, Settings, Flags, License

- `AuditEntryAppended` (optional fan-out; avoid noisy cycles)
- `AuditExportGenerated`
- `SettingsChanged`
- `FeatureFlagDefined`
- `FeatureFlagChanged`
- `LicenseActivated`
- `LicenseUpdated`
- `LicenseExpiring`
- `LicenseExpired`
- `LicenseSuspended`
- `SeatAllocated`
- `SeatReleased`
- `ModuleEntitlementChanged`

### 9.7 Event Envelope (Core Standard)

All Core events include:

- `event_id`, `event_type`, `event_version`
- `occurred_at`
- `tenant_id`
- `actor` (principal/user/system)
- `correlation_id` / `causation_id`
- `resource` (type + id)
- `payload`

---

## 10. Commands

Commands are accepted **only by Core**. Other modules never mutate Core aggregates directly.

### 10.1 Tenancy / Company / Org

| Command | Aggregate | Notes |
| --- | --- | --- |
| `ProvisionTenant` | Tenant | Bootstrap workflow |
| `SuspendTenant` | Tenant | Blocks auth |
| `ReactivateTenant` | Tenant | License must allow |
| `CloseTenant` | Tenant | Terminal |
| `RegisterCompany` | Company | |
| `UpdateCompany` | Company | |
| `DeactivateCompany` | Company | |
| `CreateOrgUnit` | OrgUnit | |
| `MoveOrgUnit` | OrgUnit | |
| `ArchiveOrgUnit` | OrgUnit | |

### 10.2 Users / Auth

| Command | Aggregate | Notes |
| --- | --- | --- |
| `InviteUser` | User | Seat check |
| `ActivateUser` | User | |
| `DeactivateUser` | User | Revoke sessions |
| `LockUser` / `UnlockUser` | User | Security |
| `LinkUserToPerson` | User | Binds Workforce person |
| `LinkExternalIdentity` | User | SSO |
| `EstablishSession` | Session | AuthN success |
| `RevokeSession` | Session | |
| `RevokeAllUserSessions` | User/Session | |

### 10.3 Access Control

| Command | Aggregate | Notes |
| --- | --- | --- |
| `DefineRole` | RoleDefinition | Tenant-custom within policy |
| `ChangeRolePermissions` | RoleDefinition | |
| `RetireRole` | RoleDefinition | |
| `GrantAccess` | AccessGrant | |
| `RevokeAccess` | AccessGrant | |

### 10.4 Membership / Teams

| Command | Aggregate | Notes |
| --- | --- | --- |
| `GrantProjectMembership` | ProjectMembership | Requires project existence (query Projects) |
| `UpdateProjectMembership` | ProjectMembership | |
| `RevokeProjectMembership` | ProjectMembership | |
| `CreateTeam` | Team | |
| `AddTeamMember` | Team | |
| `RemoveTeamMember` | Team | |
| `ArchiveTeam` | Team | |

### 10.5 Files / Audit / Config / License

| Command | Aggregate | Notes |
| --- | --- | --- |
| `CreateFileUploadIntent` | FileObject | AuthZ + license/storage limits |
| `CompleteFileUpload` | FileObject | Checksum verify |
| `QuarantineFileObject` | FileObject | Security |
| `DeleteFileObject` | FileObject | Soft/hard per policy |
| `AppendAuditEntry` | AuditStream | Open host for modules |
| `ExportAuditLog` | AuditStream | Long-running → workflow |
| `UpsertSettings` | SettingsBundle | Schema-validated |
| `DefineFeatureFlag` | FeatureFlag | Platform/admin |
| `SetFeatureFlagOverride` | FeatureFlag | |
| `ActivateLicense` | License | |
| `UpdateLicense` | License | |
| `AllocateSeat` / `ReleaseSeat` | License | |

---

## 11. Queries

Public read models / query APIs published by Core.

### 11.1 Identity & Access Queries

| Query | Result | Used by |
| --- | --- | --- |
| `GetTenant(TenantId)` | Tenant summary | All |
| `GetCompany(CompanyId)` | Company summary | Projects, Workforce |
| `GetUser(UserId)` | User profile (safe fields) | UI, modules |
| `ResolvePrincipal(SessionToken)` | Principal context | API gate |
| `Authorize(PrincipalId, PermissionCode, Scope)` | `Allow` / `Deny` + reasons | **Every command path** |
| `ListEffectivePermissions(PrincipalId, Scope?)` | Permission set | UI gating (non-authoritative alone) |
| `ListAccessGrants(PrincipalId)` | Grants | Admin |

### 11.2 Membership & Teams Queries

| Query | Result | Used by |
| --- | --- | --- |
| `IsProjectMember(ProjectId, PrincipalId\|PersonId)` | bool + roles | Projects, Safety, etc. |
| `ListProjectMembers(ProjectId)` | Membership list | Projects UI |
| `ListPrincipalProjects(PrincipalId)` | Project IDs + roles | Home, navigation |
| `GetTeam(TeamId)` | Team + members | Safety assignment UX |
| `ListTeams(scope)` | Teams | Admin / Project |

### 11.3 Files / Audit / Config / License

| Query | Result | Used by |
| --- | --- | --- |
| `GetFileObject(FileObjectId)` | Metadata | Documents, Safety, Training |
| `AuthorizeFileAccess(PrincipalId, FileObjectId, Action)` | Allow/Deny | Download/upload complete |
| `GetSettings(scope, keys)` | Effective settings | All modules |
| `EvaluateFlag(key, tenant, actor?)` | bool / variant | All modules |
| `GetLicense(TenantId)` | Entitlements | Admin, gates |
| `IsModuleEnabled(TenantId, ModuleKey)` | bool | Host routing, UI |
| `HasAvailableSeat(TenantId, SeatType)` | bool | Invite flows |
| `QueryAuditEntries(filter)` | Page of entries | Admin, COR provenance support |

### 11.4 Query Rules

- Queries return **DTOs**, never mutable aggregates.  
- UI permission hiding is UX only; server `Authorize` is mandatory.  
- Membership lists may include display snapshots; Workforce remains source of person truth.

---

## 12. Permissions Model

### 12.1 Model

```text
Allow ⇔
  Principal is active
  ∧ Tenant is active
  ∧ License allows module (when module-scoped)
  ∧ ∃ Grant(Principal, Role, Scope)
      such that Role contains PermissionCode
      and Scope covers Resource
```

### 12.2 Scope Coverage

| Resource scope | Covered by grant scope |
| --- | --- |
| Tenant-wide admin resource | Tenant scope |
| Org-bound resource | OrgUnit scope (including ancestor rules as defined) |
| Project resource | Project scope **or** Tenant admin override |
| Team resource | Team scope or parent project/tenant per policy |
| Self resource | Self scope (principal’s own user) |

### 12.3 Permission Code Catalog (Representative)

Permission codes are stable strings owned by Core, optionally **namespaced by module** for clarity—but **enforced by Core**.

**Core platform**

- `core.tenant.read`, `core.tenant.manage`
- `core.company.manage`
- `core.org.manage`
- `core.user.invite`, `core.user.manage`
- `core.role.manage`, `core.grant.manage`
- `core.membership.manage`
- `core.team.manage`
- `core.file.upload`, `core.file.read`, `core.file.delete`
- `core.audit.read`, `core.audit.export`
- `core.settings.manage`
- `core.flags.manage`
- `core.license.read`

**Downstream modules (registered in Core catalog)**

- `projects.project.create`, `projects.project.manage`, …
- `safety.activity.create`, `safety.activity.review`, `safety.action.close`, …
- `equipment.asset.manage`, `equipment.inspection.complete`, …
- `documents.document.publish`, `documents.ack.require`, …
- `signatures.package.create`, `signatures.package.void`, …
- `training.requirement.manage`, `training.completion.record`, …
- `cor.package.generate`, `cor.framework.manage`, …
- `analytics.read`, `notifications.prefer.self`, …

Module teams **propose** permission codes via platform review; Core **publishes** the catalog. Modules must not invent unchecked side-channel authz.

### 12.4 Role Types

| Kind | Description |
| --- | --- |
| **System roles** | Platform-defined baselines (Tenant Admin, Safety Admin, Supervisor, Worker, Read-only Auditor) |
| **Tenant custom roles** | Clones/edits within policy ceilings |
| **Membership roles** | Project-local role bindings used with project scope grants |

### 12.5 Defense in Depth

1. API edge calls `Authorize`  
2. Module command re-checks resource-sensitive permissions when needed  
3. License/module flag checked for gated capabilities  
4. Audit records deny on sensitive resources where policy requires  

---

## 13. Public API

Core exposes three public surfaces.

### 13.1 In-Process Application Interfaces (Modular Monolith)

Used by other Rust modules and Temporal activities:

| Interface | Capability |
| --- | --- |
| `TenancyApi` | Tenant/company/org queries |
| `IdentityApi` | User/principal resolution |
| `AuthzApi` | `authorize`, effective permissions |
| `MembershipApi` | Project membership & teams |
| `FileApi` | Upload intent, complete, metadata, access check |
| `AuditApi` | `append`, query, export request |
| `SettingsApi` | get/upsert effective settings |
| `FlagsApi` | evaluate flags |
| `LicenseApi` | module enabled, seats, license summary |

### 13.2 HTTP API (External Clients)

Versioned under `/api/core/...` (illustrative):

- Auth session/OIDC callbacks  
- Admin: users, roles, grants, companies, org, teams  
- Membership management  
- File upload intent/complete  
- Settings & flags (authorized)  
- Audit query/export  
- License read (admin)

Downstream domain HTTP APIs remain in their modules; they **call Core interfaces internally** for authz rather than trusting the client.

### 13.3 Integration Events (NATS)

Subjects namespaced e.g. `proven.core.v1.<event>` for downstream projections (notification routing, analytics, cache invalidation, COR provenance aids).

### 13.4 What Is Not Public

- Password hashes, MFA secrets  
- Internal credential tables  
- Raw grant evaluation internals  
- Cross-schema SQL access  

---

## 14. Internal Building Blocks (Within Core)

Suggested internal capability areas (still one bounded context):

```text
core/
  tenancy/          # Tenant, Company, OrgUnit
  identity/         # User, Session, credentials, SSO links
  access/           # Roles, Permissions, Grants, AuthzService
  membership/       # ProjectMembership, Team
  files/            # FileObject + R2 adapters
  audit/            # AuditStream/Entries
  settings/         # SettingsBundle
  flags/            # FeatureFlag
  licensing/        # License, seats, entitlements
  application/      # Public interfaces facade
```

These are **packages inside Core**, not separate deployable modules and not separately reachable by other domains except through the public facade.

---

## 15. How Every Other Module Communicates with Core

### 15.1 Universal Pattern

```text
Actor → Module HTTP API
      → Core.AuthzApi.Authorize(...)
      → (optional) Core membership/license/flag queries
      → Module aggregate command
      → Module persist + outbox
      → Core.AuditApi.Append(...)
      → NATS module events
```

No module bypasses Core for authn/authz/tenancy/license.

### 15.2 Per-Module Interaction Matrix

| Module | Sync calls to Core | Why | Consumes Core events |
| --- | --- | --- | --- |
| **projects** | Authz; company/tenant validation; grant/revoke membership coordination; flags/license for project limits | Project creation requires active tenant/company; membership is Core-owned | `Company*`, `User*`, `License*`, `ModuleEntitlementChanged` |
| **workforce** | Authz; link user↔person; company checks; file API for profile artifacts if any | Person is workforce-owned; login binding is Core | `UserLinkedToPerson`, `UserDeactivated` |
| **safety** | Authz; `IsProjectMember`; team lists; file upload for attachments; settings; flags; audit | Scope all activity to membership; store evidence files | `ProjectMembership*`, `Team*`, `SettingsChanged` |
| **equipment** | Authz; membership for project assignment; file API for cert binaries; audit | Asset docs as files; project scope | Membership & license events |
| **documents** | Authz; file API (bytes); settings for retention defaults; audit | Documents add *control semantics* on top of Core files | `FileObject*`, `SettingsChanged` |
| **signatures** | Authz; user/principal identity assurance; file API for signature blobs; audit | Signer identity from Core; evidence bytes via files | `User*`, `SessionRevoked` (assurance) |
| **training** | Authz; membership; file API for certificates; license seats if training-gated; audit | Completions scoped to people/projects | Membership, user deactivation |
| **cor_audit** | Authz; audit query for provenance; file API for packages; license/flags; settings | Packages stored as files; readiness gated by entitlements | `AuditExport*`, `FileObject*`, license |
| **notifications** | Authz for preference writes; user contact queries; flags; settings | Recipient resolution | `User*`, `SettingsChanged`, membership changes |
| **workflows** | Authz when starting; membership checks in activities; flags; license; audit on workflow admin | Activities call Core before domain commands | Tenant suspend, license expire |
| **analytics** | Authz for dashboards; tenant/project scope filters via membership | Never bypass RLS-equivalent scope | Many Core events for dimensions |

### 15.3 Projects ↔ Core Special Case

```text
Create Project (projects)
  → Core.Authorize(projects.project.create)
  → Core.IsModuleEnabled("projects")
  → Core.GetCompany / tenant active
  → Projects.ProjectCreated
  → Core.GrantProjectMembership(creator as PM)   # via public command
```

```text
Add worker to site
  → Core.GrantProjectMembership(...)
  → event ProjectMembershipGranted
  → Projects / Safety / Training update projections as needed
```

Projects **do not** store authoritative ACL tables that diverge from Core.

### 15.4 Documents ↔ Core Files Special Case

```text
Publish controlled document
  → Core.CreateFileUploadIntent / CompleteFileUpload
  → Documents creates DocumentVersion referencing FileObjectId
  → Documents owns effective dating & acknowledgement rules
```

Core does not know “SWP” vs “SDS.” It knows a file object and who may read it at the storage layer; Documents may impose additional domain ACL via Authz permission codes.

### 15.5 Workforce ↔ Core Users Special Case

```text
User  = who can log in (Core)
Person = who they are operationally (Workforce)
```

Link via `LinkUserToPerson`. Safety attendance references `PersonId`; AuthZ uses `PrincipalId`/`UserId`. UI joins through public queries, not SQL joins.

### 15.6 Asynchronous Collaboration

Modules primarily **react** to Core events for:

- Cache invalidation (authz, flags)  
- Notification targeting  
- Analytics dimensions  
- Disabling work when `TenantSuspended` / `LicenseExpired`  
- Cleaning local projections on membership revoke  

Modules must tolerate at-least-once delivery and use idempotent handlers.

### 15.7 Temporal Collaboration

Core may start workflows for:

- Tenant provisioning  
- Audit export  
- License expiry cascading notifications  

Compliance workflows in other modules call **Core activities** (authorize, membership check, append audit, evaluate flag) before domain activities.

### 15.8 Forbidden Communication

| Forbidden | Correct alternative |
| --- | --- |
| Module SQL join to `core.users` | `IdentityApi` / events |
| Module stores shadow password | Impossible by design |
| Module invents local admin boolean | `AuthzApi` + grants |
| Module writes R2 without FileObject | `FileApi` intent/complete |
| Module skips audit “to save time” | `AuditApi.append` required for significant commands |
| UI trusts client role claims alone | Server `Authorize` |

---

## 16. Multi-Tenancy Design

### 16.1 Isolation Model

- Every Core row is tenant-scoped (platform super-admin paths explicit and audited).  
- Downstream modules **must** include `TenantId` on owned records.  
- AuthZ and membership queries always require tenant context from the authenticated session.  
- Object keys are tenant-prefixed.  
- Feature flags and settings are tenant-overridable.  
- License is per tenant.

### 16.2 Partner Companies Inside a Tenant

A tenant may register multiple `Company` records (prime + subs). Visibility across companies is **not** automatic; it is granted through project membership and grants. This supports GC/Sub reality without separate tenants per sub (unless the sub is its own customer tenant).

### 16.3 Suspend / Expire Behavior

On `TenantSuspended` or `LicenseExpired`:

- Sessions may be revoked or limited to billing/admin read  
- Downstream write commands fail Core license/tenant gates  
- Read-only evidence access policy is product-configurable  

---

## 17. File Storage Design (Core)

### 17.1 Responsibility Split

| Layer | Owner |
| --- | --- |
| Presign, checksum, quarantine, retention class, access check | **Core** |
| Bucket/provider I/O | Infrastructure adapters (R2) |
| Document control meaning | `documents` |
| Signature evidence meaning | `signatures` |
| Safety attachment meaning | `safety` (references `FileObjectId`) |

### 17.2 Lifecycle

```text
CreateFileUploadIntent
  → authorize + license/storage limits
  → pending FileObject
  → presigned PUT
CompleteFileUpload
  → verify size/checksum/content-type
  → Available (or Quarantine if scanner signals)
Delete / Retain / Hold
  → policy + audit
```

### 17.3 Access

`AuthorizeFileAccess` combines:

- Principal grants (`core.file.*` or module permissions)  
- Object ACL / owner module claim  
- Tenant isolation  
- Quarantine state  

---

## 18. Audit Log Design (Core)

### 18.1 Requirements

- Append-only  
- Actor, action, resource, timestamp, correlation  
- Integrity digest support  
- Queryable for admin and exportable for investigations / COR provenance support  
- Mandatory for: authz changes, membership changes, license changes, file deletes, settings security keys, and any compliance-significant domain action (via `AuditApi`)

### 18.2 Module Usage

Downstream modules call `AuditApi.Append` in the same application transaction boundary when possible (or reliable outbox → Core append workflow). They do **not** keep a second authoritative audit store.

---

## 19. Settings, Feature Flags, Licensing

### 19.1 Settings

- Schema-registered keys (typed)  
- Precedence: User → OrgUnit → Tenant → Platform default  
- Modules read settings through `SettingsApi` (e.g., retention defaults, notification quiet hours policy ceilings)

### 19.2 Feature Flags

- Kill switches and progressive delivery  
- Targeting: percentage, tenant allowlist, actor  
- Modules gate optional UX/paths with `FlagsApi`  
- Flags do not replace License entitlements for paid modules

### 19.3 Licensing

- Modules, seat types, hard limits  
- Enforced on invite, module enablement, and expensive operations (export) as defined  
- Emits entitlement events so UI/host can hide disabled modules  

---

## 20. Consistency & Workflow Notes

| Operation | Consistency approach |
| --- | --- |
| Authorize on write | Sync Core query in request path |
| Membership change → module projections | Eventual via NATS |
| Tenant provision | Temporal workflow across Core commands |
| License expiry cascade | Temporal + events |
| File virus scan | Async worker updates FileObject status via Core command |
| AuthZ cache | Redis short TTL; invalidate on `Access*` / `Membership*` / `SessionRevoked` |

---

## 21. Evolution & Independence

Core evolves carefully because all modules depend on it.

**Compatibility rules:**

1. Permission codes are append-only; renames require deprecation windows.  
2. Public interface methods are versioned or additive.  
3. Events are additive.  
4. Breaking identity/tenancy changes require ADRs and migration playbooks.  
5. New foundation concerns land in Core only if truly cross-cutting; do not attract Safety rules into Core.

**Extraction path:** Core may later split into `identity` / `tenancy` services **only after** stable public interfaces exist—the current modular monolith package facade is the seam.

---

## 22. Alignment with Repository Plan

| Plan element | Mapping |
| --- | --- |
| Crate/module | `crates/modules/core` (or `proven-core`) |
| Schema | `core` |
| HTTP | `/api/core` |
| CI area label | `area:core` |
| CODEOWNERS | Platform + security co-owners |

Update the global [Domain Model](./DOMAIN_MODEL.md) catalog in a follow-on revision to replace separate `tenancy` / `identity` / `audit` entries with `core`, keeping this document as the detailed authority for Core.

---

## 23. Success Criteria

Core is correctly designed when:

1. Every module command path can answer “who / tenant / allowed?” via Core alone.  
2. No compliance module stores shadow ACLs or licenses.  
3. Files have one object identity model; documents/signatures add meaning on top.  
4. Audit can reconstruct security and membership history.  
5. Tenant suspend/license expiry cleanly stops writes platform-wide.  
6. Core can evolve its internals without other modules importing its schema.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Principal Architecture | Initial Core foundation domain design |

---

*End of Core Domain Architecture*
