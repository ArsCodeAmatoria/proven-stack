//! AuthZ integration seam (ADR-0006: "Permission codes are `users.*`, AuthZ still via
//! `AuthzApi`"). Every mutation flows through [`authorize`] or [`authorize_self_or_permission`],
//! which call the injected `proven_core::AuthzApi` — the platform's single fail-closed decision
//! path (ADR-0003).

use async_trait::async_trait;

use proven_core::application::services::{
    AuthorizeRequest, GrantAccessCommand, UpsertPermissionOverrideCommand,
};
use proven_core::domain::{
    AbacContext, AccessGrant, AccessScope, AuthzDecision, CoreError, GrantScopeType,
    PermissionOverride,
};
use proven_core::AuthzApi;
use proven_shared::{GrantId, PermissionCode, PermissionOverrideId, PrincipalId, TenantId, UserId};

/// The acting tenant + principal for a Users mutation — analogous to `CorePrincipal`.
#[derive(Debug, Clone, Copy)]
pub struct ActingContext {
    pub tenant_id: TenantId,
    pub principal: PrincipalId,
}

impl ActingContext {
    pub fn new(tenant_id: TenantId, principal: PrincipalId) -> Self {
        Self {
            tenant_id,
            principal,
        }
    }

    /// The `UserId` behind this context's principal, assuming a 1:1 principal↔user mapping
    /// (true for every Users caller today — service/system principals do not hold personal
    /// profile data).
    pub fn as_user_id(&self) -> UserId {
        UserId::from_uuid(self.principal.as_uuid())
    }
}

/// Boundary for a Users resource authorization check.
///
/// Core's `GrantScopeType` (ADR-0003) does not define a `User` scope kind, so the tightest
/// *available* boundary that still soundly covers a user-scoped resource for a third party
/// (e.g. an administrator) is `Tenant` (a Tenant-scope grant covers every resource in the
/// tenant, per `proven_core`'s `scope_covers` rule). When the acting principal *is* the target
/// user, `GrantScopeType::SelfScope` is tried first (see [`authorize_self_or_permission`]).
pub fn tenant_scope() -> AccessScope {
    AccessScope::tenant()
}

/// Calls the injected `AuthzApi` against the tenant-wide boundary, returning `Forbidden` on
/// `Deny` and mapping transport errors. Use for actions a user cannot perform on themselves
/// (profile lifecycle, kind assignment, archival) — always administrator/system-permission gated.
pub async fn authorize(
    authz: &dyn AuthzApi,
    ctx: &ActingContext,
    permission: &'static str,
) -> Result<(), crate::domain::UsersError> {
    let decision = authz
        .authorize(AuthorizeRequest {
            tenant_id: ctx.tenant_id,
            principal: ctx.principal,
            permission: PermissionCode::from(permission),
            resource: tenant_scope(),
            abac: AbacContext::empty(),
        })
        .await
        .map_err(|err| crate::domain::UsersError::Internal(format!("authz check failed: {err}")))?;

    if decision.is_allowed() {
        Ok(())
    } else {
        Err(crate::domain::UsersError::forbidden(format!(
            "missing permission: {permission}"
        )))
    }
}

/// Calls the injected `AuthzApi`, allowing the action when **either**:
/// - the acting principal *is* `target_user_id` and holds a `GrantScopeType::SelfScope` grant
///   for `permission` (own-account preference management, ADR-0006), or
/// - the acting principal holds `permission` at the tenant-wide boundary (administrator path).
///
/// Use for every account-profile mutation a user should be able to perform on their own record
/// (preferences, avatar, auth/signature profile, emergency contacts, settings, audit reads).
pub async fn authorize_self_or_permission(
    authz: &dyn AuthzApi,
    ctx: &ActingContext,
    permission: &'static str,
    target_user_id: UserId,
) -> Result<(), crate::domain::UsersError> {
    if ctx.as_user_id() == target_user_id {
        let self_scope = AccessScope {
            scope_type: GrantScopeType::SelfScope,
            scope_id: Some(target_user_id.as_uuid()),
        };
        let self_decision = authz
            .authorize(AuthorizeRequest {
                tenant_id: ctx.tenant_id,
                principal: ctx.principal,
                permission: PermissionCode::from(permission),
                resource: self_scope,
                abac: AbacContext::empty(),
            })
            .await
            .map_err(|err| {
                crate::domain::UsersError::Internal(format!("authz check failed: {err}"))
            })?;
        if self_decision.is_allowed() {
            return Ok(());
        }
    }

    authorize(authz, ctx, permission).await
}

/// Stub `AuthzApi` that always allows — used by `UsersServices::in_memory_unchecked()` for unit
/// tests that don't need to exercise real AuthZ wiring. Production and integration paths should
/// use `UsersServices::with_core`, which wires the real `proven-core` `AuthzApi`.
pub struct AllowAllAuthz;

#[async_trait]
impl AuthzApi for AllowAllAuthz {
    async fn authorize(&self, _req: AuthorizeRequest) -> Result<AuthzDecision, CoreError> {
        Ok(AuthzDecision::allow("users_stub_allow_all"))
    }

    async fn grant_access(&self, _cmd: GrantAccessCommand) -> Result<AccessGrant, CoreError> {
        Err(CoreError::Internal(
            "AllowAllAuthz does not support grant_access".into(),
        ))
    }

    async fn revoke_access(
        &self,
        _tenant_id: TenantId,
        _grant_id: GrantId,
        _revoked_by: Option<UserId>,
    ) -> Result<(), CoreError> {
        Err(CoreError::Internal(
            "AllowAllAuthz does not support revoke_access".into(),
        ))
    }

    async fn list_effective_permissions(
        &self,
        _tenant_id: TenantId,
        _principal: PrincipalId,
    ) -> Result<Vec<PermissionCode>, CoreError> {
        Ok(Vec::new())
    }

    async fn upsert_permission_override(
        &self,
        _cmd: UpsertPermissionOverrideCommand,
    ) -> Result<PermissionOverride, CoreError> {
        Err(CoreError::Internal(
            "AllowAllAuthz does not support upsert_permission_override".into(),
        ))
    }

    async fn revoke_permission_override(
        &self,
        _tenant_id: TenantId,
        _id: PermissionOverrideId,
        _revoked_by: Option<UserId>,
    ) -> Result<(), CoreError> {
        Err(CoreError::Internal(
            "AllowAllAuthz does not support revoke_permission_override".into(),
        ))
    }

    async fn list_permission_overrides(
        &self,
        _tenant_id: TenantId,
        _user_id: UserId,
    ) -> Result<Vec<PermissionOverride>, CoreError> {
        Ok(Vec::new())
    }
}
