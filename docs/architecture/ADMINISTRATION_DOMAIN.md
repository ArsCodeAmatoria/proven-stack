# Proven — Administration Domain Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Administration Domain Architecture (DDD) |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Principal Architecture |
| **Audience** | Engineering, Product, Design, Security, Customer Admins |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Core Domain](./CORE_DOMAIN.md), [Projects Domain](./PROJECTS_DOMAIN.md), [System Architecture](./SYSTEM_ARCHITECTURE.md), [UX Architecture](../ux/UX_ARCHITECTURE.md), [Repository Plan](./REPOSITORY_PLAN.md), [PRD](../PRD.md) |

---

## 1. Purpose

This document defines the **Administration** bounded context for Proven.

Administration is the **operator control plane** of the Construction Compliance Operating System: the desktop-first console where company admins configure tenants, access, projects (orchestration), branding, builders, integrations, licensing visibility, audit review, API keys, and system health.

Administration is primarily a **composition / facade domain**. It owns admin-specific configuration surfaces (branding, API keys, integration registrations, admin dashboard layout, builder studio metadata, system health views, future billing shells). It does **not** fork Core’s SoR for users, roles, permissions, flags, licenses, or audit entries—nor Projects’ SoR for project lifecycle.

**Documentation only — no implementation.**

---

## 2. Bounded Context

### 2.1 Context Card

| Field | Value |
| --- | --- |
| **Name** | Administration |
| **Module** | `admin` |
| **Strategic type** | Generic / platform supporting subdomain |
| **Product metaphor** | Admin = configure the OS safely without entering field workflows |
| **System of record for** | Tenant branding, admin console preferences, API keys/clients, integration registrations & connector bindings (metadata), workflow/template *builder drafts & publications registry* (coordination), administration dashboard definitions, system health snapshots/views, billing account stubs (future) |
| **Not system of record for** | Companies/users/roles/permissions/flags/licenses/audit streams (**Core**); project aggregates (**Projects**); domain templates content (**Projects/Safety/Documents/Training** owning modules); notification templates (**Notifications**); Temporal runtime (**Workflows** platform) |

### 2.2 Context Map

```text
┌──────────────────────────────────────────────┐
│              ADMINISTRATION                  │
│  Console · Branding · API Keys · Integrations│
│  Builders registry · Health · Billing stub   │
└───────────────┬──────────────────────────────┘
                │ orchestrates via public APIs
    ┌───────────┼───────────┬────────────┬────────────┐
    ▼           ▼           ▼            ▼            ▼
  Core      Projects    Workflows   Notifications  Analytics
 (identity,  (projects,  (defs)      (templates)    (exec views)
  flags,     templates)
  license,
  audit)
```

### 2.3 Ubiquitous Language

| Term | Meaning |
| --- | --- |
| **Admin Console** | Desktop administration application area |
| **Company Management** | Admin UX over Core companies/orgs |
| **Builder** | Visual authoring studio for workflows or templates |
| **Publication** | Promoting a builder draft into the owning module’s live definition |
| **API Key** | Machine credential for integrations |
| **Integration** | Registered external system connection |
| **Branding** | Tenant visual identity for web/PWA/guest surfaces |
| **System Health** | Degraded dependency & job health visibility for admins |
| **Licensing Panel** | Read/manage view over Core license entitlements |
| **Billing** (future) | Commercial account, invoices, payment methods |

---

## 3. Responsibilities (Mapped to Ownership)

| Responsibility | Admin owns? | Clarification |
| --- | --- | --- |
| **Company Management** | UX + orchestration | **Core** owns Company/OrgUnit aggregates |
| **Projects** | UX + orchestration | **Projects** owns project lifecycle; Admin lists/creates via Projects API |
| **Users** | UX + orchestration | **Core** owns User/Session |
| **Roles** | UX + orchestration | **Core** owns RoleDefinition |
| **Permissions** | UX + catalog views | **Core** owns permission catalog & grants |
| **Feature Flags** | UX + orchestration | **Core** owns FeatureFlag |
| **Branding** | Yes | Logos, colors, guest/sign-in chrome |
| **Workflow Builder** | Studio + registry | Published definitions live in **Workflows**; Admin owns drafts/UI studio metadata |
| **Template Builder** | Studio + registry | Published templates live in owning modules (Projects/Safety/Documents/Training/Notifications); Admin owns cross-cutting studio UX & draft registry |
| **Audit Logs** | UX + query orchestration | **Core** owns AuditStream/entries |
| **API Keys** | Yes | Key records + hashed secrets |
| **Integrations** | Yes (registry) | Connector config metadata; secrets in platform secret store |
| **Licensing** | UX + orchestration | **Core** owns License aggregate |
| **Billing** | Future Yes | Stub aggregates only until commercial launch |
| **System Health** | Yes (views/snapshots) | Aggregates probes from platform; not a second APM product |
| **Administration Dashboard** | Yes | Admin home composition |

---

## 4. Aggregate Roots

| Aggregate | Responsibility |
| --- | --- |
| **TenantBranding** | Brand assets, theme tokens, guest/login presentation |
| **AdminConsoleSettings** | Admin UI prefs, default landing sections |
| **ApiClient** | OAuth client / API key principal metadata |
| **ApiKey** | Hashed key material, scopes, expiry, rotation |
| **IntegrationRegistration** | External system type, status, bound endpoints |
| **BuilderDraft** | In-progress workflow/template draft document |
| **BuilderPublication** | Record of publish to owning module + version link |
| **AdminDashboardDefinition** | Widgets for admin home |
| **SystemHealthSnapshot** | Periodic health summary for tenant/platform |
| **BillingAccount** *(future)* | Customer billing profile stub |
| **AdminAuditViewPreference** | Saved filters for audit log UX (not audit data) |

---

## 5. Entities

### 5.1 Branding & Console

| Entity | Description |
| --- | --- |
| **BrandAssetRef** | Logo/wordmark `FileObjectId` |
| **ThemeTokenSet** | Color/typography tokens (constrained) |
| **GuestChromeConfig** | Guest signing header/footer |
| **AdminNavCustomization** | Optional section ordering |

### 5.2 API Keys & Integrations

| Entity | Description |
| --- | --- |
| **ApiKeyScopeBinding** | Granted permission scopes (subset of Core catalog) |
| **ApiKeyRotation** | Rotation history |
| **IntegrationCapability** | What the integration can do (webhooks, HRIS sync…) |
| **WebhookEndpoint** | Inbound/outbound URL metadata |
| **ConnectorSecretRef** | Pointer to secret store id—not raw secret |
| **SyncCursor** | Optional integration watermark |

### 5.3 Builders

| Entity | Description |
| --- | --- |
| **DraftDocument** | JSON/DSL body for workflow or template |
| **ValidationResult** | Builder lint/validation outcomes |
| **PublicationTarget** | Module + resource type (ProjectTemplate, ActivityType, …) |
| **VersionPin** | Published foreign id/version |

### 5.4 Health & Dashboard

| Entity | Description |
| --- | --- |
| **HealthCheckResult** | Dependency status (API, workers, NATS, Temporal, CH, R2…) |
| **QueueDepthStat** | Notification DLQ / ingest lag summaries |
| **AdminWidget** | License seats used, flag status, failing checks, recent admin actions |

### 5.5 Billing (Future)

| Entity | Description |
| --- | --- |
| **BillingContact** | Accounts payable contact |
| **SubscriptionPlanRef** | Link to commercial plan |
| **InvoiceStub** | Placeholder until billing provider integrated |

---

## 6. Value Objects

- `BrandingId`, `ApiClientId`, `ApiKeyId`, `IntegrationId`
- `BuilderDraftId`, `BuilderKind` — Workflow | ProjectTemplate | SafetyActivityType | DocumentTemplate | TrainingCourse | NotificationTemplate | CORPack…
- `DraftStatus` — Editing | Validated | Submitted | Published | Discarded
- `PublicationStatus` — Succeeded | Failed | RolledBack
- `ApiKeyStatus` — Active | Rotating | Revoked | Expired
- `IntegrationStatus` — Connected | Degraded | Disconnected | Disabled
- `HealthStatus` — Healthy | Degraded | Unhealthy | Unknown
- `ThemeTokens`, `HexColor`, `FileObjectId`
- `ScopeCode` (permission codes allowed on keys)
- `SecretRef`, `WebhookUrl`
- `SeatUtilization` — { used, limit, seatType } from Core license query

---

## 7. Relationships

```text
Tenant (Core)
  ├── TenantBranding (Admin)
  ├── AdminConsoleSettings (Admin)
  ├── ApiClient 1──* ApiKey (Admin) ──scopes──► Core Permission catalog
  ├── IntegrationRegistration (Admin) ──may use──► ApiClient
  ├── BuilderDraft ──publishes──► BuilderPublication
  │         └── target APIs: Workflows / Projects / Safety / Documents / Training / Notifications / COR
  ├── AdminDashboardDefinition (Admin)
  └── SystemHealthSnapshot (Admin)

Admin Console UX ──commands/queries──►
    Core (companies, users, roles, grants, flags, license, audit)
    Projects (projects, project templates)
    Analytics (optional exec tiles)
```

### 7.1 Critical Boundary Rule

```text
Admin UI “Create User”
  → Admin application service
  → Core.InviteUser / GrantAccess
  → Core events + audit
  → Admin does NOT persist user rows
```

Same pattern for companies, roles, flags, licenses, projects.

---

## 8. Administration Dashboard

### 8.1 Purpose

Admin Home answers: “Is the tenant configured, licensed, healthy, and secure enough to operate Proven?”

Not a field Command Center ([UX](../ux/UX_ARCHITECTURE.md)).

### 8.2 Widget Blocks

| Block | Data source |
| --- | --- |
| **License & seats** | Core LicenseApi |
| **Module entitlements** | Core flags/license |
| **Users & access risk** | Core (invites pending, admins count, keys expiring) |
| **Integrations health** | Admin IntegrationRegistration + probes |
| **System health** | SystemHealthSnapshot |
| **Audit highlights** | Core Audit query (recent security events) |
| **Builder drafts** | Unpublished BuilderDrafts awaiting publish |
| **Projects summary** | Projects list counts (orchestration) |
| **Billing** (future) | BillingAccount status |

### 8.3 Rules

1. Prefer actionable admin tasks (rotate key, fix connector, publish draft).  
2. No field KPI wallpaper (that belongs to Analytics/Command Center).  
3. All widgets respect admin permissions.  
4. Deep links into Core/Projects admin subpages—single console IA.

### 8.4 Wireframe (Logical)

```text
┌─ Administration ────────────────────────────────────────────┐
│ Proven Admin · Acme Construction                            │
│ [Dashboard] Companies Users Access Projects Branding …      │
│                                                             │
│ License: Active · 412/500 seats        Modules: 8 enabled   │
│ Health: Degraded · Notifications DLQ 12                     │
│                                                             │
│ Needs you                                                   │
│ · 3 API keys expiring in 7d                                 │
│ · Teams connector disconnected                              │
│ · 2 workflow drafts ready to publish                        │
│                                                             │
│ Recent security audit                          View logs →  │
│ · AccessGranted · RoleChanged · ApiKeyRevoked               │
└─────────────────────────────────────────────────────────────┘
```

---

## 9. Domain Events (Admin-Owned)

- `TenantBrandingUpdated`
- `AdminConsoleSettingsChanged`
- `ApiClientCreated` / `Updated`
- `ApiKeyIssued` / `Rotated` / `Revoked` / `Expired`
- `IntegrationRegistered` / `Connected` / `Degraded` / `Disconnected`
- `BuilderDraftCreated` / `Updated` / `Discarded`
- `BuilderPublishRequested` / `BuilderPublishSucceeded` / `BuilderPublishFailed`
- `AdminDashboardDefinitionChanged`
- `SystemHealthSnapshotTaken`
- `BillingAccountUpdated` *(future)*

When Admin orchestrates Core/Projects, **those modules’ events** remain authoritative (`UserInvited`, `ProjectCreated`, …).

---

## 10. Business Rules

### 10.1 Facade Integrity

1. Admin never duplicates Core user/role/grant tables.  
2. Admin never duplicates Projects write models.  
3. All mutating admin actions that affect security call Core and append Core Audit.  
4. Failed Core calls → no partial Admin “shadow success.”

### 10.2 Branding

1. Assets via Core FileApi; Admin stores refs + tokens.  
2. Tokens constrained to brand-safe ranges (accessibility contrast checks recommended).  
3. Branding applies to web/PWA/guest chrome—not to Core AuthZ.  
4. Changes audited.

### 10.3 API Keys

1. Raw keys shown **once** at issue; only hashes stored.  
2. Scopes ⊆ caller’s grantable set and tenant policy ceilings.  
3. Expiry required for production keys (policy).  
4. Rotation issues new key; old grace window optional then revoke.  
5. Keys authenticate as service principals in Core Identity (link).  
6. High-risk scopes (audit export, void signatures) require step-up + dual control optional.

### 10.4 Integrations

1. Registry is SoR for connection metadata; secrets only as `SecretRef`.  
2. Disable integration immediately on security incident.  
3. Inbound webhooks verify signatures; map to module commands via ACL.  
4. Integrations must not bypass AuthZ or audit.

### 10.5 Workflow & Template Builders

1. Drafts are Admin-owned until publish.  
2. Publish validates then calls owning module command (`PublishProjectTemplate`, `DefineActivityType`, `DefineWorkflowDefinition`, …).  
3. Owning module remains SoR for live definitions.  
4. Rollback = owning module retire/version + Admin publication record.  
5. Builders cannot encode rules that violate module invariants—validation is module-side authoritative.  
6. Cross-module “mega templates” publish as coordinated Temporal onboarding workflow, not a single denormalized blob SoR in Admin.

### 10.6 Audit Log Viewer

1. Read-only over Core AuditApi.  
2. Filters saved in Admin preferences only.  
3. Export uses Core audit export workflows; Admin triggers, does not copy ledger.

### 10.7 Feature Flags & Licensing Panels

1. Toggle/override via Core Flags/License APIs.  
2. Admin UI must show effective license vs flag (flags ≠ paid entitlement).  
3. Seat utilization widgets query Core—no local counters as truth.

### 10.8 System Health

1. Snapshots are informational for tenant admins (subset) and platform ops (full).  
2. Health never auto-remediates domain data.  
3. Sensitive infra detail hidden from customer admins (provider internals).

### 10.9 Billing (Future)

1. Billing must not gate field safety writes without explicit product policy; prefer license grace from Core.  
2. Until launch, only stubs/flags.

### 10.10 Desktop-Only Primary

Per UX: Administration is desktop-first; mobile shows limited account settings only.

---

## 11. Permissions

Administration permissions are registered in Core and checked via AuthZ. Many screens also require underlying Core/Projects permissions.

### 11.1 Admin Module Codes

| Code | Intent |
| --- | --- |
| `admin.console.access` | Enter Administration area |
| `admin.dashboard.read` | Admin home |
| `admin.branding.manage` | Branding |
| `admin.apikey.manage` | Issue/rotate/revoke keys |
| `admin.integration.manage` | Integrations |
| `admin.builder.edit` | Edit builder drafts |
| `admin.builder.publish` | Publish drafts to modules |
| `admin.health.read` | System health (tenant level) |
| `admin.health.read_platform` | Full platform health (ops) |
| `admin.billing.manage` | Future billing |
| `admin.audit.view` | Open audit viewer (also needs `core.audit.read`) |

### 11.2 Orchestrated Codes (Examples)

| Screen | Also requires |
| --- | --- |
| Companies | `core.company.manage` / read |
| Users | `core.user.*` |
| Roles & grants | `core.role.manage`, `core.grant.manage` |
| Feature flags | `core.flags.manage` |
| Licensing | `core.license.read` (+ manage if any) |
| Projects admin | `projects.project.*` |
| Audit export | `core.audit.export` |

### 11.3 Separation of Duties (Recommended Policy)

- `admin.builder.publish` ≠ alone sufficient for production Safety type changes without `safety.*` manage  
- Key managers ≠ automatic tenant super-admins  
- Billing admins ≠ security admins (future)

---

## 12. Public Interfaces & API (Summary)

### 12.1 Admin-Owned APIs

Base: `/api/admin`

- `/dashboard`
- `/branding`
- `/api-clients`, `/api-keys`
- `/integrations`
- `/builders/drafts`, `/builders/publish`
- `/health`
- `/billing` (future stub)

### 12.2 Orchestrated Proxies (Optional BFF-style)

Admin UI may call Core/Projects APIs directly from the web app **or** via thin Admin gateway routes that only forward. Prefer **direct module APIs** to avoid Admin becoming a god adapter—gateway only when aggregation is required (dashboard).

### 12.3 In-Process

| Interface | Purpose |
| --- | --- |
| `AdminDashboardApi` | Compose admin home DTO |
| `BrandingApi` | Get effective branding for web/guest |
| `ApiKeyApi` | Issue/verify metadata (verify via Core Identity integration) |
| `IntegrationRegistryApi` | List connectors for workers |
| `BuilderRegistryApi` | Drafts/publications |
| `SystemHealthApi` | Snapshots |

---

## 13. Workflow Integration

| Workflow | Purpose |
| --- | --- |
| `TenantAdminOnboardingWorkflow` | Company + admin user + branding defaults + license bootstrap (calls Core) |
| `BuilderPublishWorkflow` | Validate → publish to module(s) → record publication |
| `ApiKeyExpiryWorkflow` | Warn/revoke expired keys + notify |
| `IntegrationHealthPollWorkflow` | Update IntegrationStatus |
| `SystemHealthProbeWorkflow` | Write SystemHealthSnapshot |
| `BillingSyncWorkflow` *(future)* | Provider sync |

---

## 14. Notifications & Audit

- Admin events (key expiry, connector down, publish failed) → Notifications.  
- All security-sensitive Admin actions → **Core Audit** (and Admin module events where owned).  
- Viewing audit logs is itself auditable for sensitive exports.

---

## 15. Administration IA (Console Nav)

```text
Administration
├── Dashboard
├── Organization
│   ├── Companies
│   └── Org Units
├── Access
│   ├── Users
│   ├── Roles & Permissions
│   └── Project Memberships (via Core/Projects UX)
├── Projects (admin list/create)
├── Configuration
│   ├── Feature Flags
│   ├── Licensing
│   └── Branding
├── Builders
│   ├── Workflow Builder
│   └── Template Builder
├── Security
│   ├── API Keys
│   ├── Integrations
│   └── Audit Logs
├── System Health
└── Billing (future)
```

---

## 16. Data Ownership

### 16.1 Schema `admin` Owns

- Branding, console settings  
- API clients/keys (hashed)  
- Integration registrations  
- Builder drafts & publication records  
- Admin dashboard definitions  
- Health snapshots  
- Billing stubs (future)  

### 16.2 Forbidden Duplication

| Do not store as SoR in Admin | Owner |
| --- | --- |
| Users, roles, grants, sessions | Core |
| Feature flag evaluations authority | Core |
| License seats authority | Core |
| Audit entries | Core |
| Projects, participants | Projects |
| Live workflow definitions | Workflows |
| Live safety/doc/training templates | Owning modules |

---

## 17. Security Considerations

1. Admin console behind strong AuthZ + optional SSO + step-up for keys/integrations.  
2. Least privilege admin roles out of the box.  
3. API keys hashed; rotation mandatory practices.  
4. Integration secrets never in git or Admin plaintext fields.  
5. Builder publish is change-managed (who/when/what target).  
6. Customer admins see tenant-scoped health only.  
7. Branding uploads scanned via Core file pipeline.

---

## 18. Anti-Patterns

1. Admin database copying Core users “for convenience”  
2. Publishing templates without module-side validation  
3. Unlimited-scope API keys  
4. Using Admin as the field operations app  
5. Treating feature flags as billing entitlement  
6. Hiding audit export without Core audit  
7. Billing hard-cut that blocks sealed safety evidence submission without grace policy  

---

## 19. Success Criteria

Administration is correctly designed when:

1. Admins can configure the tenant end-to-end from one console IA.  
2. Core/Projects remain authoritative for identity and places.  
3. Branding, keys, and integrations are safely owned and audited.  
4. Builders accelerate configuration without forking domain SoR.  
5. Admin Dashboard surfaces license, access risk, connector, and health actions—not field vanity KPIs.  
6. Future billing plugs in without rewriting the facade model.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Principal Architecture | Initial Administration facade/control-plane domain |

---

*End of Administration Domain Architecture*
