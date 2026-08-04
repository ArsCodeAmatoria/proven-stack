//! AuthZ integration seam (ADR-0005 §5: "Permission codes are `companies.*`, AuthZ still via
//! `AuthzApi`"). Every mutation flows through [`authorize`], which calls the injected
//! `proven_core::AuthzApi` — the platform's single fail-closed decision path (ADR-0003).

use async_trait::async_trait;

use proven_core::application::services::{
    AuthorizeRequest, GrantAccessCommand, UpsertPermissionOverrideCommand,
};
use proven_core::domain::{
    AbacContext, AccessGrant, AccessScope, AuthzDecision, CoreError, GrantScopeType,
    PermissionOverride,
};
use proven_core::AuthzApi;
use proven_shared::{
    CompanyId, GrantId, PermissionCode, PermissionOverrideId, PrincipalId, TenantId, UserId,
};

/// The acting tenant + principal for a Companies mutation — analogous to `CorePrincipal`.
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
}

/// Boundary for a Companies resource authorization check.
///
/// Core's `GrantScopeType` (ADR-0003) does not yet define a `Company` scope kind, so the
/// tightest *available* boundary that still soundly covers a company-scoped resource is
/// `Tenant` (a Tenant-scope grant covers every resource in the tenant, per
/// `proven_core`'s `scope_covers` rule). When Core adds a dedicated Company scope, this is the
/// only place that needs to change.
pub fn company_scope(_company_id: CompanyId) -> AccessScope {
    AccessScope {
        scope_type: GrantScopeType::Tenant,
        scope_id: None,
    }
}

/// Calls the injected `AuthzApi`, returning `Forbidden` on `Deny` and mapping transport errors.
pub async fn authorize(
    authz: &dyn AuthzApi,
    ctx: &ActingContext,
    permission: &'static str,
    company_id: CompanyId,
) -> Result<(), crate::domain::CompaniesError> {
    let decision = authz
        .authorize(AuthorizeRequest {
            tenant_id: ctx.tenant_id,
            principal: ctx.principal,
            permission: PermissionCode::from(permission),
            resource: company_scope(company_id),
            abac: AbacContext::empty(),
        })
        .await
        .map_err(|err| {
            crate::domain::CompaniesError::Internal(format!("authz check failed: {err}"))
        })?;

    if decision.is_allowed() {
        Ok(())
    } else {
        Err(crate::domain::CompaniesError::forbidden(format!(
            "missing permission: {permission}"
        )))
    }
}

/// Stub `AuthzApi` that always allows — used by `CompaniesServices::in_memory_unchecked()` for
/// unit tests that don't need to exercise real AuthZ wiring. Production and integration paths
/// should use `CompaniesServices::with_core`, which wires the real `proven-core` `AuthzApi`.
pub struct AllowAllAuthz;

#[async_trait]
impl AuthzApi for AllowAllAuthz {
    async fn authorize(&self, _req: AuthorizeRequest) -> Result<AuthzDecision, CoreError> {
        Ok(AuthzDecision::allow("companies_stub_allow_all"))
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
