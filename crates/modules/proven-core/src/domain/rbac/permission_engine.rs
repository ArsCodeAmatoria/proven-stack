//! `PermissionEngine` — the fail-closed evaluation core behind `AuthzApi::authorize`
//! (ADR-0003, ADR-0007 §2). Pure and I/O-free: `AuthzService` loads grants/roles/overrides and
//! hands them here.
//!
//! ## Evaluation order (fail closed)
//!
//! 1. `!module_enabled` → deny `module_disabled` (license/feature precondition, ADR-0007 §7).
//! 2. An active **deny** override covering `(permission, scope)` → deny `override_deny`.
//! 3. An active **allow** override covering `(permission, scope)` → allow `override_allow`
//!    (grants without a role at all — explicit emergency/temporary access).
//! 4. An active grant whose role has the permission and whose scope covers the resource → allow.
//! 5. Otherwise → deny `no_covering_grant`.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use proven_shared::{PermissionCode, RoleId};
use uuid::Uuid;

use crate::domain::authz::{AccessScope, AuthzDecision};
use crate::domain::enums::GrantScopeType;
use crate::domain::models::{AccessGrant, PermissionOverride, RoleDefinition};

/// Everything [`evaluate`] needs to decide one `(principal, permission, resource)` triple.
pub struct EvaluationInput<'a> {
    pub permission: &'a PermissionCode,
    pub resource: AccessScope,
    pub principal_user_id: Uuid,
    pub grants: &'a [AccessGrant],
    pub roles: &'a HashMap<RoleId, RoleDefinition>,
    pub overrides: &'a [PermissionOverride],
    pub now: DateTime<Utc>,
    /// License/feature precondition already resolved by the caller (ADR-0007 §7: feature
    /// gating is a precondition inside the engine, not a second RBAC system).
    pub module_enabled: bool,
}

/// Coverage rule shared by grants and overrides (CORE_DOMAIN.md §12.2, ADR-0007 §4): tenant
/// scope covers everything; self scope covers only the principal's own user record; every other
/// scope kind (including the new `Company`) requires an exact `(scope_type, scope_id)` match —
/// so a `Company` grant never covers a `Project` resource, and vice versa, unless the grant is
/// actually `Tenant`-scoped.
pub fn scope_covers(grant_scope: &AccessScope, resource: &AccessScope, own_user_id: Uuid) -> bool {
    match grant_scope.scope_type {
        GrantScopeType::Tenant => true,
        GrantScopeType::SelfScope => {
            resource.scope_type == GrantScopeType::SelfScope
                && resource
                    .scope_id
                    .map(|id| id == own_user_id)
                    .unwrap_or(true)
        }
        _ => {
            grant_scope.scope_type == resource.scope_type
                && match (grant_scope.scope_id, resource.scope_id) {
                    (Some(g), Some(r)) => g == r,
                    (Some(_), None) => false,
                    (None, _) => true,
                }
        }
    }
}

fn override_covers(
    override_: &PermissionOverride,
    permission: &PermissionCode,
    resource: &AccessScope,
    principal_user_id: Uuid,
    now: DateTime<Utc>,
) -> bool {
    override_.is_active(now)
        && &override_.permission_code == permission
        && scope_covers(&override_.scope, resource, principal_user_id)
}

/// Namespace for the pure evaluation entry point — kept as a unit struct (rather than a bare
/// free function) so it reads symmetrically with [`super::role_engine::RoleEngine`] at call
/// sites (`PermissionEngine::evaluate(input)`).
pub struct PermissionEngine;

impl PermissionEngine {
    pub fn evaluate(input: EvaluationInput<'_>) -> AuthzDecision {
        evaluate(input)
    }
}

/// Run the fail-closed RBAC decision described in the module docs.
pub fn evaluate(input: EvaluationInput<'_>) -> AuthzDecision {
    if !input.module_enabled {
        return AuthzDecision::deny("module_disabled");
    }

    if let Some(deny) = input.overrides.iter().find(|o| {
        o.effect == crate::domain::enums::OverrideEffect::Deny
            && override_covers(
                o,
                input.permission,
                &input.resource,
                input.principal_user_id,
                input.now,
            )
    }) {
        return AuthzDecision::deny(format!("override_deny:{}", deny.id));
    }

    if let Some(allow) = input.overrides.iter().find(|o| {
        o.effect == crate::domain::enums::OverrideEffect::Allow
            && override_covers(
                o,
                input.permission,
                &input.resource,
                input.principal_user_id,
                input.now,
            )
    }) {
        return AuthzDecision::allow(format!("override_allow:{}", allow.id));
    }

    for grant in input.grants.iter().filter(|g| g.is_active(input.now)) {
        let Some(role) = input.roles.get(&grant.role_id) else {
            continue;
        };
        if role.has_permission(input.permission)
            && scope_covers(&grant.scope, &input.resource, input.principal_user_id)
        {
            return AuthzDecision::allow(format!("grant:{} role:{}", grant.id, role.name));
        }
    }

    AuthzDecision::deny("no_covering_grant")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::enums::{GrantKind, OverrideEffect, RoleKind, RoleStatus};
    use proven_shared::{GrantId, PermissionOverrideId, TenantId, UserId};

    fn role(id: RoleId, permissions: &[&str]) -> RoleDefinition {
        RoleDefinition {
            id,
            tenant_id: None,
            name: "Test Role".to_string(),
            kind: RoleKind::System,
            status: RoleStatus::Active,
            permissions: permissions.iter().map(|p| (*p).into()).collect(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version: 1,
        }
    }

    fn grant(role_id: RoleId, scope: AccessScope, user_id: UserId) -> AccessGrant {
        AccessGrant {
            id: GrantId::new(),
            tenant_id: TenantId::new(),
            user_id,
            role_id,
            scope,
            grant_kind: GrantKind::Standard,
            expires_at: None,
            revoked_at: None,
            created_at: Utc::now(),
            created_by: None,
        }
    }

    #[test]
    fn denies_when_module_disabled_even_with_covering_grant() {
        let user_id = UserId::new();
        let role_id = RoleId::new();
        let mut roles = HashMap::new();
        roles.insert(role_id, role(role_id, &["documents.document.read"]));
        let grants = vec![grant(role_id, AccessScope::tenant(), user_id)];

        let decision = evaluate(EvaluationInput {
            permission: &"documents.document.read".into(),
            resource: AccessScope::tenant(),
            principal_user_id: user_id.as_uuid(),
            grants: &grants,
            roles: &roles,
            overrides: &[],
            now: Utc::now(),
            module_enabled: false,
        });
        assert!(!decision.is_allowed());
    }

    #[test]
    fn deny_override_wins_over_covering_grant() {
        let user_id = UserId::new();
        let role_id = RoleId::new();
        let mut roles = HashMap::new();
        roles.insert(role_id, role(role_id, &["core.user.manage"]));
        let grants = vec![grant(role_id, AccessScope::tenant(), user_id)];
        let overrides = vec![PermissionOverride {
            id: PermissionOverrideId::new(),
            tenant_id: TenantId::new(),
            user_id,
            permission_code: "core.user.manage".into(),
            effect: OverrideEffect::Deny,
            scope: AccessScope::tenant(),
            reason: None,
            expires_at: None,
            revoked_at: None,
            created_at: Utc::now(),
            created_by: None,
        }];

        let decision = evaluate(EvaluationInput {
            permission: &"core.user.manage".into(),
            resource: AccessScope::tenant(),
            principal_user_id: user_id.as_uuid(),
            grants: &grants,
            roles: &roles,
            overrides: &overrides,
            now: Utc::now(),
            module_enabled: true,
        });
        assert!(!decision.is_allowed());
    }

    #[test]
    fn allow_override_grants_without_any_role() {
        let user_id = UserId::new();
        let overrides = vec![PermissionOverride {
            id: PermissionOverrideId::new(),
            tenant_id: TenantId::new(),
            user_id,
            permission_code: "core.user.manage".into(),
            effect: OverrideEffect::Allow,
            scope: AccessScope::tenant(),
            reason: Some("emergency access".to_string()),
            expires_at: None,
            revoked_at: None,
            created_at: Utc::now(),
            created_by: None,
        }];

        let decision = evaluate(EvaluationInput {
            permission: &"core.user.manage".into(),
            resource: AccessScope::tenant(),
            principal_user_id: user_id.as_uuid(),
            grants: &[],
            roles: &HashMap::new(),
            overrides: &overrides,
            now: Utc::now(),
            module_enabled: true,
        });
        assert!(decision.is_allowed());
    }

    #[test]
    fn company_scope_does_not_cover_project_scope() {
        let user_id = UserId::new();
        let role_id = RoleId::new();
        let mut roles = HashMap::new();
        roles.insert(role_id, role(role_id, &["core.company.manage"]));
        let company_id = Uuid::new_v4();
        let grants = vec![grant(role_id, AccessScope::company(company_id), user_id)];

        let project_resource = AccessScope::project(Uuid::new_v4());
        let decision = evaluate(EvaluationInput {
            permission: &"core.company.manage".into(),
            resource: project_resource,
            principal_user_id: user_id.as_uuid(),
            grants: &grants,
            roles: &roles,
            overrides: &[],
            now: Utc::now(),
            module_enabled: true,
        });
        assert!(!decision.is_allowed());

        let company_resource = AccessScope::company(company_id);
        let decision = evaluate(EvaluationInput {
            permission: &"core.company.manage".into(),
            resource: company_resource,
            principal_user_id: user_id.as_uuid(),
            grants: &grants,
            roles: &roles,
            overrides: &[],
            now: Utc::now(),
            module_enabled: true,
        });
        assert!(decision.is_allowed());
    }

    #[test]
    fn expired_grant_denies() {
        let user_id = UserId::new();
        let role_id = RoleId::new();
        let mut roles = HashMap::new();
        roles.insert(role_id, role(role_id, &["core.user.manage"]));
        let mut expired_grant = grant(role_id, AccessScope::tenant(), user_id);
        expired_grant.expires_at = Some(Utc::now() - chrono::Duration::seconds(1));

        let decision = evaluate(EvaluationInput {
            permission: &"core.user.manage".into(),
            resource: AccessScope::tenant(),
            principal_user_id: user_id.as_uuid(),
            grants: &[expired_grant],
            roles: &roles,
            overrides: &[],
            now: Utc::now(),
            module_enabled: true,
        });
        assert!(!decision.is_allowed());
    }
}
