# ADR-0007: Enterprise RBAC in Core (ABAC-ready)

| Field | Value |
| --- | --- |
| Status | Accepted |
| Date | 2026-08-03 |
| Deciders | Lead Software Engineering, Security |

## Context

Proven requires enterprise RBAC: system/company/project/temporary roles, permission overrides, and module permission families (documents, equipment, training, safety, approvals, features). Architecture forbids parallel AuthZ systems ([AUTHORIZATION_RBAC_ARCHITECTURE.md](../architecture/AUTHORIZATION_RBAC_ARCHITECTURE.md), ADR-0003).

## Decision

1. **Extend `proven-core` AuthZ** — do not create a separate RBAC crate or module schema.
2. Introduce explicit **`RoleEngine`** and **`PermissionEngine`** used by `AuthzApi` / `AuthorizationService`.
3. Expand role kinds: `system`, `company`, `project`, `temporary`, plus existing `tenant_custom` / `membership`.
4. Expand grant scopes with **`company`** (alongside tenant, org_unit, project, team, self).
5. Add **`permission_overrides`** (allow/deny, optional expiry) evaluated after grants; deny overrides win.
6. Publish module permission catalogs into `core.permissions` (future modules consume codes today).
7. **Feature / license gating** remains a precondition inside the PermissionEngine (Flags + License ports), not a second RBAC.
8. **Authorization policies** compose engine steps; include an **`AbacContext`** (resource attributes, assurance) that policies may inspect — default policy ignores unknown attrs (ABAC-ready without enforcing full ABAC yet).
9. Platform **AuthZ middleware** requires a permission + scope for protected routes; fail closed.

## Consequences

- JWT still never carries permission lists.
- Temporary roles use `RoleKind::Temporary` and/or `GrantKind::Temporary` with `expires_at`.
- ABAC attribute enforcement lands as additional `AuthorizationPolicy` implementations later.
