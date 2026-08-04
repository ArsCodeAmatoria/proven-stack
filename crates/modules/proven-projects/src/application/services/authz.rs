//! AuthZ integration seam — every mutation flows through [`authorize`] → `AuthzApi` (ADR-0003).

use async_trait::async_trait;

use proven_core::application::services::{
    AuthorizeRequest, GrantAccessCommand, UpsertPermissionOverrideCommand,
};
use proven_core::domain::{
    AbacContext, AccessGrant, AccessScope, AuthzDecision, CoreError, PermissionOverride,
};
use proven_core::AuthzApi;
use proven_shared::{
    GrantId, PermissionCode, PermissionOverrideId, PrincipalId, ProjectId, TenantId, UserId,
};

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

pub fn tenant_scope() -> AccessScope {
    AccessScope::tenant()
}

pub fn project_scope(project_id: ProjectId) -> AccessScope {
    AccessScope::project(project_id.as_uuid())
}

pub async fn authorize(
    authz: &dyn AuthzApi,
    ctx: &ActingContext,
    permission: &'static str,
    resource: AccessScope,
) -> Result<(), crate::domain::ProjectsError> {
    let decision = authz
        .authorize(AuthorizeRequest {
            tenant_id: ctx.tenant_id,
            principal: ctx.principal,
            permission: PermissionCode::from(permission),
            resource,
            abac: AbacContext::empty(),
        })
        .await
        .map_err(|err| {
            crate::domain::ProjectsError::Internal(format!("authz check failed: {err}"))
        })?;

    if decision.is_allowed() {
        Ok(())
    } else {
        Err(crate::domain::ProjectsError::forbidden(format!(
            "missing permission: {permission}"
        )))
    }
}

/// Stub AuthZ that always allows — for `ProjectsServices::in_memory_unchecked()`.
pub struct AllowAllAuthz;

#[async_trait]
impl AuthzApi for AllowAllAuthz {
    async fn authorize(&self, _req: AuthorizeRequest) -> Result<AuthzDecision, CoreError> {
        Ok(AuthzDecision::allow("projects_stub_allow_all"))
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
