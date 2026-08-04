# Enterprise RBAC (developer notes)

Canonical design: [ADR-0007](../adr/0007-enterprise-rbac.md) and
[AUTHORIZATION_RBAC_ARCHITECTURE.md](../architecture/AUTHORIZATION_RBAC_ARCHITECTURE.md).

This extends the existing Core AuthZ system (`crates/modules/proven-core`) — it is **not** a
parallel RBAC implementation. `AuthzApi` (ADR-0003) remains the only decision authority; every
module keeps calling `AuthzApi::authorize(...)` exactly as before. What changed is what happens
*inside* `AuthzService::authorize`.

## Mental model

```text
Allow iff
  tenant active ∧ principal active
  ∧ policies.before_rbac(...) does not deny   (ABAC short-circuit, e.g. sealed resource)
  ∧ module_enabled(permission)                (license/feature precondition)
  ∧ no active DENY override covers (permission, scope)
  ∧ ( an active ALLOW override covers (permission, scope)
      ∨ an active grant's role has permission ∧ grant.scope covers scope )
  ∧ policies.after_allow(...) does not revoke
```

Fail closed at every step: missing data, disabled module, or no covering grant/override → deny.

## Engines (`src/domain/rbac/`)

### `RoleEngine`

Pure helpers around role kinds and grant lifecycle — no I/O:

- `is_system_role(kind)` — `System`, `Company`, `Project`, `Temporary` are all platform-shipped
  role kinds (they ship with the permission catalog); `TenantCustom` and `Membership` are
  tenant-authored.
- `requires_expiry_for_role_kind` / `requires_expiry_for_grant_kind` — `Temporary` roles and
  `Temporary`/`BreakGlass` grants must carry `expires_at`.
- `validate_expiry(role_kind, grant_kind, expires_at)` — **hard** validation used by
  `grant_access`; rejects temporary access without an expiry.
- `validate_role_for_scope(kind, scope)` — **soft** validation (a warning, not a rejection):
  `Company` roles expect `Company` scope, `Project` roles expect `Project`/`Team` scope.
  `AuthzService::grant_access` logs a `tracing::warn!` when this fires but still creates the
  grant — some legitimate grants intentionally cross this (e.g. a small tenant's owner holding
  Company Admin at Tenant scope).
- `system_role_ids()` / `is_system_role_id(id)` — the fixed UUIDs matching
  `db/migrations/core/20260803200001_core_permissions_seed.sql` and
  `db/migrations/core/20260803230001_core_enterprise_rbac_seed.sql` (`domain::permissions`).

### `PermissionEngine`

The pure evaluation core. `AuthzService` loads grants/roles/overrides from the repository ports
and hands them to `PermissionEngine::evaluate(EvaluationInput { .. }) -> AuthzDecision`:

1. `!module_enabled` → deny `module_disabled`.
2. An active **deny** override covering `(permission, scope)` → deny `override_deny` (deny
   always wins).
3. An active **allow** override covering `(permission, scope)` → allow `override_allow` (this is
   the only path that can allow *without* any role — emergency/temporary access).
4. An active grant whose role has the permission and whose scope covers the resource → allow.
5. Otherwise → deny `no_covering_grant`.

`scope_covers` (moved here from the old `authz_service.rs`) is the single source of truth for
scope coverage: `Tenant` covers everything; `Self` covers only the principal's own user record;
every other scope kind — `OrgUnit`, **`Company`**, `Project`, `Team` — requires an exact
`(scope_type, scope_id)` match. **A `Company` grant never covers a `Project` resource, or vice
versa** — only a `Tenant`-scoped grant crosses that boundary.

### Policies (ABAC-ready, not ABAC-enforcing)

`AuthorizationPolicy` is the composition point for attribute-based rules around the RBAC engine:

```rust
trait AuthorizationPolicy {
    fn before_rbac(&self, ctx: &AbacContext, permission: &PermissionCode) -> Option<AuthzDecision>;
    fn after_allow(&self, ctx: &AbacContext, permission: &PermissionCode) -> Option<AuthzDecision>;
}
```

`AuthzService` runs a fixed chain (`SealedResourcePolicy`, then `DefaultRbacPolicy`) around
`PermissionEngine::evaluate`. `before_rbac` can short-circuit to `Deny` before grants/overrides
are even loaded; `after_allow` only runs when RBAC would otherwise allow, and can still revoke
that allow.

`AbacContext` carries the (currently mostly-unused) ABAC inputs:

- `resource_attributes: HashMap<String, String>` — reserved for module-contributed dimensions
  (classification, restricted flags, …). Empty today.
- `assurance_level: Option<String>` — step-up/MFA signal for future elevated-permission checks.
- `resource_state: Option<String>` — e.g. `"sealed"`, `"draft"`, `"published"`.

`SealedResourcePolicy` is the first concrete consumer: if `resource_state == "sealed"` and the
permission's action segment contains `.manage`, `.publish`, or `.void`, it denies
(`resource_sealed`) regardless of role — sealed-evidence immutability
(AUTHORIZATION_RBAC_ARCHITECTURE.md §1 rule 5). Future modules add more `AuthorizationPolicy`
implementations here without touching `PermissionEngine` or `AuthzApi`.

## Scopes

`GrantScopeType` / `AccessScope`: `Tenant`, `OrgUnit`, `Company`, `Project`, `Team`, `Self`.
`AccessScope::company(company_id)` joins the existing `tenant()`, `project(id)`,
`self_scope(user_id)` constructors.

## Role kinds

`RoleKind`: `System`, `TenantCustom`, `Membership`, `Company`, `Project`, `Temporary`. The nine
system roles (`domain::permissions`) mirror the SQL seed:

| Role | Kind | Fixed UUID helper |
| --- | --- | --- |
| Tenant Admin | `System` | `system_tenant_admin_role_id()` |
| Company Admin | `Company` | `company_admin_role_id()` |
| Project Admin | `Project` | `project_admin_role_id()` |
| Supervisor | `Project` | `supervisor_role_id()` |
| Worker | `Project` | `worker_role_id()` |
| Safety Coordinator | `Project` | `safety_coordinator_role_id()` |
| Equipment Manager | `Company` | `equipment_manager_role_id()` |
| Training Admin | `Company` | `training_admin_role_id()` |
| Document Control | `Company` | `document_control_role_id()` |
| Temporary Elevated | `Temporary` | `temporary_elevated_role_id()` |

## Permission overrides

`core.permission_overrides` (`domain::models::PermissionOverride`): an explicit allow/deny for a
single `(tenant, user, permission, scope)`, with optional `reason` and `expires_at`. Deny always
wins over both allow overrides and role grants. Managed through `AuthzApi`:

- `upsert_permission_override(cmd) -> PermissionOverride`
- `revoke_permission_override(tenant_id, id, revoked_by)`
- `list_permission_overrides(tenant_id, user_id) -> Vec<PermissionOverride>`

HTTP surface: `POST /api/v1/core/authz/overrides`, `DELETE
/api/v1/core/authz/overrides/{id}`, `GET /api/v1/core/authz/overrides?user_id=...`. Every
mutation is audited (`core.override.created` / `core.override.revoked`) and published on the
outbox (`PermissionOverrideCreated` / `PermissionOverrideRevoked`), matching the grant/revoke
pattern.

`GET /api/v1/core/roles` is a thin catalog-browse endpoint listing the platform-shipped system
roles (tenant custom roles aren't listable yet — `RoleRepository` only supports point lookups).

## Feature / license gating

`AuthzService::module_enabled` checks the permission's leading segment (`documents.*`,
`equipment.*`, `training.*`, `safety.*`, `approvals.*`, `projects.*`, `feature.*` —
`domain::permissions::LICENSE_GATED_MODULE_PREFIXES`) against `LicenseApi::is_module_enabled`.
Permission codes outside that list (`core.*`, and anything else) are never license-gated — Core
itself is foundational. This is a **precondition inside** `PermissionEngine`, not a second RBAC
system (ADR-0007 §7).

## Platform middleware

`crates/proven-platform/src/http/middleware/authz.rs` exports:

- `AuthzPrincipal` — the acting tenant/user from `X-Proven-Tenant-Id` / `X-Proven-User-Id`
  (same interim scheme as `proven_core::api::extractors::CorePrincipal` — ADR-0002).
- `require_permission(state, principal, permission, scope) -> Result<(), ApiError>` — calls
  `state.core().services.authorize(...)` and maps `Deny` to `403 Forbidden`.

Any module's Axum handlers can call `require_permission` at the top of a protected handler; it
never re-implements RBAC, it only adapts `AuthzApi`'s decision to an HTTP response.

## Testing

`crates/modules/proven-core/tests/enterprise_rbac_tests.rs` exercises the engines end-to-end
through `CoreModule::in_memory()` (seeded via `MemoryStore::seeded()`, which now includes the
nine system roles above) plus a few direct `PermissionEngine`/`SealedResourcePolicy` unit tests
colocated with the engines themselves (`src/domain/rbac/*.rs`).
