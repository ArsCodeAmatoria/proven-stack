//! Authorization policies — composable steps around [`super::permission_engine::PermissionEngine`]
//! (ADR-0007 §8, AUTHORIZATION_RBAC_ARCHITECTURE.md §17).
//!
//! ## ABAC-ready, not ABAC-enforcing
//!
//! [`AbacContext`] carries the attribute-based-access-control inputs Proven will eventually
//! enforce broadly (resource attributes, assurance level, resource state). Today only
//! [`SealedResourcePolicy`] inspects it. Future modules add more `AuthorizationPolicy`
//! implementations without touching `PermissionEngine` or `AuthzApi` — the composition point is
//! `AuthzService::authorize` (`before_rbac` runs first and can short-circuit deny; `after_allow`
//! runs only when RBAC would otherwise allow, and can still revoke to deny).

use std::collections::HashMap;

use proven_shared::PermissionCode;

use crate::domain::authz::AuthzDecision;

/// Attribute-based context accompanying an authorization request. Empty/`None` fields mean "no
/// ABAC signal available" — policies must treat that as non-blocking (fail closed only on an
/// explicit rule match, never on missing ABAC data).
#[derive(Debug, Clone, Default)]
pub struct AbacContext {
    /// Free-form resource attributes (classification, restricted flags, …). Empty today —
    /// reserved for module-contributed ABAC dimensions.
    pub resource_attributes: HashMap<String, String>,
    /// Authentication assurance signal (e.g. `"mfa"`) for step-up-gated permissions.
    pub assurance_level: Option<String>,
    /// Resource lifecycle state (e.g. `"sealed"`, `"draft"`, `"published"`) — see
    /// AUTHORIZATION_RBAC_ARCHITECTURE.md §17 "State" dimension.
    pub resource_state: Option<String>,
}

impl AbacContext {
    pub fn empty() -> Self {
        Self::default()
    }
}

/// A composable authorization policy evaluated around the core RBAC decision.
pub trait AuthorizationPolicy: Send + Sync {
    fn name(&self) -> &str;

    /// Runs before [`super::permission_engine::evaluate`]. Returning `Some(Deny)` short-circuits
    /// the whole authorization call (fail closed) without consulting grants/overrides.
    fn before_rbac(&self, ctx: &AbacContext, permission: &PermissionCode) -> Option<AuthzDecision>;

    /// Runs only when RBAC produced an `Allow`. Returning `Some(Deny)` revokes that allow.
    fn after_allow(&self, ctx: &AbacContext, permission: &PermissionCode) -> Option<AuthzDecision>;
}

/// No-op policy: documents the "plain RBAC" baseline explicitly rather than leaving the policy
/// chain implicit.
pub struct DefaultRbacPolicy;

impl AuthorizationPolicy for DefaultRbacPolicy {
    fn name(&self) -> &str {
        "default_rbac"
    }

    fn before_rbac(
        &self,
        _ctx: &AbacContext,
        _permission: &PermissionCode,
    ) -> Option<AuthzDecision> {
        None
    }

    fn after_allow(
        &self,
        _ctx: &AbacContext,
        _permission: &PermissionCode,
    ) -> Option<AuthzDecision> {
        None
    }
}

/// Sealed-evidence immutability (AUTHORIZATION_RBAC_ARCHITECTURE.md §1 rule 5): once a resource
/// is `sealed`, mutating actions are denied regardless of role — only read/withdraw-adjacent
/// actions remain possible. A permission "looks like" a mutation when its action segment
/// contains `.manage`, `.publish`, or `.void`.
pub struct SealedResourcePolicy;

impl SealedResourcePolicy {
    fn is_mutating_action(permission: &PermissionCode) -> bool {
        let code = permission.as_str();
        code.contains(".manage") || code.contains(".publish") || code.contains(".void")
    }
}

impl AuthorizationPolicy for SealedResourcePolicy {
    fn name(&self) -> &str {
        "sealed_resource"
    }

    fn before_rbac(&self, ctx: &AbacContext, permission: &PermissionCode) -> Option<AuthzDecision> {
        let sealed = ctx.resource_state.as_deref() == Some("sealed");
        if sealed && Self::is_mutating_action(permission) {
            return Some(AuthzDecision::deny("resource_sealed"));
        }
        None
    }

    fn after_allow(
        &self,
        _ctx: &AbacContext,
        _permission: &PermissionCode,
    ) -> Option<AuthzDecision> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sealed_resource_policy_denies_manage_action() {
        let policy = SealedResourcePolicy;
        let ctx = AbacContext {
            resource_state: Some("sealed".to_string()),
            ..AbacContext::empty()
        };
        let permission = PermissionCode::from("documents.document.manage");
        let decision = policy.before_rbac(&ctx, &permission);
        assert!(matches!(decision, Some(AuthzDecision::Deny { .. })));
    }

    #[test]
    fn sealed_resource_policy_allows_read_action() {
        let policy = SealedResourcePolicy;
        let ctx = AbacContext {
            resource_state: Some("sealed".to_string()),
            ..AbacContext::empty()
        };
        let permission = PermissionCode::from("documents.document.read");
        assert!(policy.before_rbac(&ctx, &permission).is_none());
    }

    #[test]
    fn sealed_resource_policy_ignores_non_sealed_state() {
        let policy = SealedResourcePolicy;
        let ctx = AbacContext {
            resource_state: Some("draft".to_string()),
            ..AbacContext::empty()
        };
        let permission = PermissionCode::from("documents.document.manage");
        assert!(policy.before_rbac(&ctx, &permission).is_none());
    }
}
