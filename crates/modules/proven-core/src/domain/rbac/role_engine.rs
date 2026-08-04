//! `RoleEngine` — pure rules about role kinds, scopes, and expiry (ADR-0007 §3).
//!
//! No I/O: `AuthzService` calls these helpers around repository reads/writes so the rules stay
//! independently testable and reusable by future admin tooling (e.g. role-catalog UIs).

use proven_shared::RoleId;

use crate::domain::authz::AccessScope;
use crate::domain::enums::{GrantKind, GrantScopeType, RoleKind};
use crate::domain::permissions;

/// Namespace for the pure role rules below — mirrors [`super::permission_engine::PermissionEngine`].
pub struct RoleEngine;

impl RoleEngine {
    pub fn is_system_role(kind: RoleKind) -> bool {
        is_system_role(kind)
    }

    pub fn requires_expiry_for_role_kind(kind: RoleKind) -> bool {
        requires_expiry_for_role_kind(kind)
    }

    pub fn requires_expiry_for_grant_kind(kind: GrantKind) -> bool {
        requires_expiry_for_grant_kind(kind)
    }

    pub fn validate_expiry(
        role_kind: RoleKind,
        grant_kind: GrantKind,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), String> {
        validate_expiry(role_kind, grant_kind, expires_at)
    }

    pub fn validate_role_for_scope(kind: RoleKind, scope: &AccessScope) -> Option<String> {
        validate_role_for_scope(kind, scope)
    }

    pub fn system_role_ids() -> Vec<RoleId> {
        system_role_ids()
    }

    pub fn is_system_role_id(id: RoleId) -> bool {
        is_system_role_id(id)
    }
}

/// Platform-shipped role kinds — ship with the permission catalog, not arbitrarily deleted
/// (AUTHORIZATION_RBAC_ARCHITECTURE.md §5.1 "System roles"). `TenantCustom` and `Membership`
/// are tenant-authored, so they are *not* system roles.
pub fn is_system_role(kind: RoleKind) -> bool {
    matches!(
        kind,
        RoleKind::System | RoleKind::Company | RoleKind::Project | RoleKind::Temporary
    )
}

/// Whether a role of this kind must only ever be granted with an `expires_at` (ADR-0007
/// consequence: "Temporary roles use `RoleKind::Temporary` ... with `expires_at`").
pub fn requires_expiry_for_role_kind(kind: RoleKind) -> bool {
    matches!(kind, RoleKind::Temporary)
}

/// Whether a grant of this kind must carry an `expires_at` (temporary + break-glass access is
/// time-bounded by definition — AUTHORIZATION_RBAC_ARCHITECTURE.md §16).
pub fn requires_expiry_for_grant_kind(kind: GrantKind) -> bool {
    matches!(kind, GrantKind::Temporary | GrantKind::BreakGlass)
}

/// Hard validation: a temporary role or temporary/break-glass grant kind without `expires_at`
/// is a programming/API error, not just a style warning — reject it outright (fail closed).
pub fn validate_expiry(
    role_kind: RoleKind,
    grant_kind: GrantKind,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<(), String> {
    if (requires_expiry_for_role_kind(role_kind) || requires_expiry_for_grant_kind(grant_kind))
        && expires_at.is_none()
    {
        return Err(format!(
            "temporary access (role kind {role_kind:?}, grant kind {grant_kind:?}) requires expires_at"
        ));
    }
    Ok(())
}

/// Soft validation: some role kinds have a natural scope. Mismatches are not rejected outright
/// (a Company Admin *could* legitimately be granted at Tenant scope for a small operator) but
/// are surfaced so callers can warn/audit (task: "prefers Company scope (warn/validate)").
pub fn validate_role_for_scope(kind: RoleKind, scope: &AccessScope) -> Option<String> {
    match kind {
        RoleKind::Company if scope.scope_type != GrantScopeType::Company => Some(format!(
            "role kind Company typically expects Company scope, got {:?}",
            scope.scope_type
        )),
        RoleKind::Project
            if !matches!(
                scope.scope_type,
                GrantScopeType::Project | GrantScopeType::Team
            ) =>
        {
            Some(format!(
                "role kind Project typically expects Project or Team scope, got {:?}",
                scope.scope_type
            ))
        }
        _ => None,
    }
}

/// Every platform-shipped system role id (tenant-independent), matching
/// `db/migrations/core/20260803200001_core_permissions_seed.sql` and
/// `db/migrations/core/20260803230001_core_enterprise_rbac_seed.sql`.
pub fn system_role_ids() -> Vec<RoleId> {
    vec![
        permissions::system_tenant_admin_role_id(),
        permissions::company_admin_role_id(),
        permissions::project_admin_role_id(),
        permissions::supervisor_role_id(),
        permissions::worker_role_id(),
        permissions::safety_coordinator_role_id(),
        permissions::equipment_manager_role_id(),
        permissions::training_admin_role_id(),
        permissions::document_control_role_id(),
        permissions::temporary_elevated_role_id(),
    ]
}

pub fn is_system_role_id(id: RoleId) -> bool {
    system_role_ids().contains(&id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn temporary_role_kind_requires_expiry() {
        assert!(requires_expiry_for_role_kind(RoleKind::Temporary));
        assert!(!requires_expiry_for_role_kind(RoleKind::Company));
    }

    #[test]
    fn validate_expiry_rejects_missing_expiry_for_temporary_grant_kind() {
        let err = validate_expiry(RoleKind::System, GrantKind::Temporary, None);
        assert!(err.is_err());

        let ok = validate_expiry(RoleKind::System, GrantKind::Temporary, Some(Utc::now()));
        assert!(ok.is_ok());
    }

    #[test]
    fn validate_role_for_scope_warns_on_mismatch() {
        let warning = validate_role_for_scope(RoleKind::Company, &AccessScope::tenant());
        assert!(warning.is_some());

        let ok = validate_role_for_scope(
            RoleKind::Company,
            &AccessScope::company(uuid::Uuid::new_v4()),
        );
        assert!(ok.is_none());
    }

    #[test]
    fn system_role_ids_include_company_admin() {
        assert!(is_system_role_id(permissions::company_admin_role_id()));
        assert!(!is_system_role_id(RoleId::new()));
    }
}
