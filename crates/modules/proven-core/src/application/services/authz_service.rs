//! `AuthorizationService` — the only fail-closed AuthZ decision path (ADR-0003, ADR-0007,
//! CORE_DOMAIN.md §12.1-§12.2). See `docs/development/ENTERPRISE_RBAC.md` for the full picture:
//! `RoleEngine` + `PermissionEngine` + `AuthorizationPolicy` chain, composed here and nowhere
//! else — modules must call this through `AuthzApi`, never re-implement RBAC.

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use proven_shared::{
    GrantId, PermissionCode, PermissionOverrideId, PrincipalId, RoleId, TenantId, UserId,
};

use crate::application::ports::{
    AuditRepository, EventPublisher, GrantRepository, LicenseRepository, OverrideRepository,
    RoleRepository, TenantRepository, UserRepository,
};
use crate::application::services::audit_service::{AppendAuditEntryCommand, AuditService};
use crate::application::services::license_service::LicenseService;
use crate::domain::permissions::LICENSE_GATED_MODULE_PREFIXES;
use crate::domain::{
    AbacContext, AccessGrant, AccessScope, AuthorizationPolicy, AuthzDecision, CoreError,
    DefaultRbacPolicy, EvaluationInput, GrantKind, OverrideEffect, PermissionEngine,
    PermissionOverride, RoleDefinition, RoleEngine, SealedResourcePolicy, TenantStatus,
    UserStatus,
};
use crate::events::{ActorRef, CoreEvent, EventEnvelope, ResourceRef};

/// One `AuthzApi::authorize` call. `abac` defaults to [`AbacContext::empty`] — callers that have
/// no ABAC signal (most today) simply omit it via `..Default::default()`-style construction is
/// not possible (tenant/principal/permission/resource have no sensible defaults), so build with
/// `abac: AbacContext::empty()` explicitly, or use [`AuthorizeRequest::new`].
pub struct AuthorizeRequest {
    pub tenant_id: TenantId,
    pub principal: PrincipalId,
    pub permission: PermissionCode,
    pub resource: AccessScope,
    pub abac: AbacContext,
}

impl AuthorizeRequest {
    /// Convenience constructor for the common case of no ABAC signal.
    pub fn new(
        tenant_id: TenantId,
        principal: PrincipalId,
        permission: PermissionCode,
        resource: AccessScope,
    ) -> Self {
        Self {
            tenant_id,
            principal,
            permission,
            resource,
            abac: AbacContext::empty(),
        }
    }
}

pub struct GrantAccessCommand {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub role_id: RoleId,
    pub scope: AccessScope,
    pub grant_kind: GrantKind,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_by: Option<UserId>,
}

pub struct UpsertPermissionOverrideCommand {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub permission: PermissionCode,
    pub effect: OverrideEffect,
    pub scope: AccessScope,
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_by: Option<UserId>,
}

pub struct AuthzService {
    tenants: Arc<dyn TenantRepository>,
    users: Arc<dyn UserRepository>,
    roles: Arc<dyn RoleRepository>,
    grants: Arc<dyn GrantRepository>,
    overrides: Arc<dyn OverrideRepository>,
    license: Arc<dyn LicenseRepository>,
    audit: Arc<dyn AuditRepository>,
    outbox: Arc<dyn EventPublisher>,
}

/// Module key gate a permission code falls under, if any (ADR-0007 §7 / AUTHORIZATION_RBAC_
/// ARCHITECTURE.md §8). Only the module prefixes future feature-gated modules use are checked —
/// `core.*` (and any other prefix) is never license-gated, since Core itself is foundational.
fn license_gated_module(permission: &PermissionCode) -> Option<proven_shared::ModuleKey> {
    let prefix = permission.as_str().split('.').next()?;
    LICENSE_GATED_MODULE_PREFIXES
        .contains(&prefix)
        .then(|| proven_shared::ModuleKey(prefix.to_string()))
}

impl AuthzService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenants: Arc<dyn TenantRepository>,
        users: Arc<dyn UserRepository>,
        roles: Arc<dyn RoleRepository>,
        grants: Arc<dyn GrantRepository>,
        overrides: Arc<dyn OverrideRepository>,
        license: Arc<dyn LicenseRepository>,
        audit: Arc<dyn AuditRepository>,
        outbox: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            tenants,
            users,
            roles,
            grants,
            overrides,
            license,
            audit,
            outbox,
        }
    }

    /// Fixed policy chain (ADR-0007 §8). Future modules add policies here — the chain, not
    /// `PermissionEngine`, is the ABAC extension point.
    fn policies() -> Vec<Box<dyn AuthorizationPolicy>> {
        vec![Box::new(SealedResourcePolicy), Box::new(DefaultRbacPolicy)]
    }

    async fn module_enabled(
        &self,
        tenant_id: TenantId,
        permission: &PermissionCode,
    ) -> Result<bool, CoreError> {
        match license_gated_module(permission) {
            Some(module) => {
                LicenseService::new(self.license.clone())
                    .is_module_enabled(tenant_id, &module)
                    .await
            }
            None => Ok(true),
        }
    }

    /// `Allow ⇔ Principal active ∧ Tenant active ∧ module_enabled ∧ policies pass ∧
    /// PermissionEngine::evaluate(...) = Allow`. Fail closed on any error or missing data.
    pub async fn authorize(&self, req: AuthorizeRequest) -> Result<AuthzDecision, CoreError> {
        let tenant = match self.tenants.get(req.tenant_id).await? {
            Some(t) => t,
            None => return Ok(AuthzDecision::deny("tenant_not_found")),
        };
        if tenant.status != TenantStatus::Active {
            return Ok(AuthzDecision::deny("tenant_not_active"));
        }

        let user_id = UserId::from_uuid(req.principal.as_uuid());
        let user = match self.users.get(req.tenant_id, user_id).await? {
            Some(u) => u,
            None => return Ok(AuthzDecision::deny("principal_not_found")),
        };
        if user.status != UserStatus::Active {
            return Ok(AuthzDecision::deny("user_not_active"));
        }

        let policies = Self::policies();
        for policy in &policies {
            if let Some(decision) = policy.before_rbac(&req.abac, &req.permission) {
                return Ok(decision);
            }
        }

        let module_enabled = self.module_enabled(req.tenant_id, &req.permission).await?;

        let now = Utc::now();
        let grants = self.grants.list_for_user(req.tenant_id, user_id).await?;
        let overrides = self.overrides.list_for_user(req.tenant_id, user_id).await?;

        let mut roles: HashMap<RoleId, RoleDefinition> = HashMap::new();
        for grant in grants.iter().filter(|g| g.is_active(now)) {
            if roles.contains_key(&grant.role_id) {
                continue;
            }
            if let Some(role) = self.roles.get(grant.role_id).await? {
                roles.insert(role.id, role);
            }
        }

        let mut decision = PermissionEngine::evaluate(EvaluationInput {
            permission: &req.permission,
            resource: req.resource,
            principal_user_id: user.id.as_uuid(),
            grants: &grants,
            roles: &roles,
            overrides: &overrides,
            now,
            module_enabled,
        });

        if decision.is_allowed() {
            for policy in &policies {
                if let Some(revoked) = policy.after_allow(&req.abac, &req.permission) {
                    decision = revoked;
                    break;
                }
            }
        }

        Ok(decision)
    }

    pub async fn grant_access(&self, cmd: GrantAccessCommand) -> Result<AccessGrant, CoreError> {
        let role = self
            .roles
            .get(cmd.role_id)
            .await?
            .ok_or_else(|| CoreError::validation("role does not exist"))?;

        RoleEngine::validate_expiry(role.kind, cmd.grant_kind, cmd.expires_at)
            .map_err(CoreError::validation)?;

        if let Some(warning) = RoleEngine::validate_role_for_scope(role.kind, &cmd.scope) {
            tracing::warn!(
                role_id = %role.id,
                role_kind = ?role.kind,
                "grant_access scope mismatch: {warning}"
            );
        }

        let grant = AccessGrant {
            id: GrantId::new(),
            tenant_id: cmd.tenant_id,
            user_id: cmd.user_id,
            role_id: cmd.role_id,
            scope: cmd.scope,
            grant_kind: cmd.grant_kind,
            expires_at: cmd.expires_at,
            revoked_at: None,
            created_at: Utc::now(),
            created_by: cmd.created_by,
        };
        self.grants.insert(&grant).await?;

        AuditService::new(self.audit.clone())
            .append(AppendAuditEntryCommand {
                tenant_id: cmd.tenant_id,
                actor_user_id: cmd.created_by,
                actor_type: "user".to_string(),
                action: "core.access.granted".to_string(),
                resource_type: "access_grant".to_string(),
                resource_id: Some(grant.id.as_uuid()),
                correlation_id: None,
                causation_id: None,
                payload: serde_json::json!({
                    "user_id": grant.user_id,
                    "role_id": grant.role_id,
                }),
                category: Some("authz".to_string()),
                ..Default::default()
            })
            .await?;

        self.outbox
            .publish(EventEnvelope::new(
                cmd.tenant_id,
                cmd.created_by
                    .map(|user_id| ActorRef::User { user_id })
                    .unwrap_or(ActorRef::System),
                ResourceRef {
                    resource_type: "access_grant".to_string(),
                    resource_id: grant.id.as_uuid(),
                },
                None,
                None,
                CoreEvent::AccessGranted {
                    tenant_id: cmd.tenant_id,
                    grant_id: grant.id,
                    user_id: grant.user_id,
                    role_id: grant.role_id,
                },
            ))
            .await?;

        Ok(grant)
    }

    pub async fn revoke_access(
        &self,
        tenant_id: TenantId,
        grant_id: GrantId,
        revoked_by: Option<UserId>,
    ) -> Result<(), CoreError> {
        if self.grants.get(tenant_id, grant_id).await?.is_none() {
            return Err(CoreError::NotFound("access_grant"));
        }
        self.grants.revoke(tenant_id, grant_id, Utc::now()).await?;

        AuditService::new(self.audit.clone())
            .append(AppendAuditEntryCommand {
                tenant_id,
                actor_user_id: revoked_by,
                actor_type: "user".to_string(),
                action: "core.access.revoked".to_string(),
                resource_type: "access_grant".to_string(),
                resource_id: Some(grant_id.as_uuid()),
                correlation_id: None,
                causation_id: None,
                payload: serde_json::json!({}),
                category: Some("authz".to_string()),
                ..Default::default()
            })
            .await?;

        self.outbox
            .publish(EventEnvelope::new(
                tenant_id,
                revoked_by
                    .map(|user_id| ActorRef::User { user_id })
                    .unwrap_or(ActorRef::System),
                ResourceRef {
                    resource_type: "access_grant".to_string(),
                    resource_id: grant_id.as_uuid(),
                },
                None,
                None,
                CoreEvent::AccessRevoked {
                    tenant_id,
                    grant_id,
                },
            ))
            .await?;

        Ok(())
    }

    pub async fn list_effective_permissions(
        &self,
        tenant_id: TenantId,
        principal: PrincipalId,
    ) -> Result<Vec<PermissionCode>, CoreError> {
        let user_id = UserId::from_uuid(principal.as_uuid());
        let now = Utc::now();
        let grants = self.grants.list_for_user(tenant_id, user_id).await?;

        let mut codes = Vec::new();
        for grant in grants.iter().filter(|g| g.is_active(now)) {
            if let Some(role) = self.roles.get(grant.role_id).await? {
                for permission in role.permissions {
                    if !codes.contains(&permission) {
                        codes.push(permission);
                    }
                }
            }
        }
        Ok(codes)
    }

    pub async fn upsert_permission_override(
        &self,
        cmd: UpsertPermissionOverrideCommand,
    ) -> Result<PermissionOverride, CoreError> {
        let override_ = PermissionOverride {
            id: PermissionOverrideId::new(),
            tenant_id: cmd.tenant_id,
            user_id: cmd.user_id,
            permission_code: cmd.permission,
            effect: cmd.effect,
            scope: cmd.scope,
            reason: cmd.reason,
            expires_at: cmd.expires_at,
            revoked_at: None,
            created_at: Utc::now(),
            created_by: cmd.created_by,
        };
        self.overrides.insert(&override_).await?;

        AuditService::new(self.audit.clone())
            .append(AppendAuditEntryCommand {
                tenant_id: cmd.tenant_id,
                actor_user_id: cmd.created_by,
                actor_type: "user".to_string(),
                action: "core.override.created".to_string(),
                resource_type: "permission_override".to_string(),
                resource_id: Some(override_.id.as_uuid()),
                correlation_id: None,
                causation_id: None,
                payload: serde_json::json!({
                    "user_id": override_.user_id,
                    "permission_code": override_.permission_code,
                    "effect": override_.effect,
                }),
                category: Some("authz".to_string()),
                ..Default::default()
            })
            .await?;

        self.outbox
            .publish(EventEnvelope::new(
                cmd.tenant_id,
                cmd.created_by
                    .map(|user_id| ActorRef::User { user_id })
                    .unwrap_or(ActorRef::System),
                ResourceRef {
                    resource_type: "permission_override".to_string(),
                    resource_id: override_.id.as_uuid(),
                },
                None,
                None,
                CoreEvent::PermissionOverrideCreated {
                    tenant_id: cmd.tenant_id,
                    override_id: override_.id,
                    user_id: override_.user_id,
                    permission_code: override_.permission_code.clone(),
                },
            ))
            .await?;

        Ok(override_)
    }

    pub async fn revoke_permission_override(
        &self,
        tenant_id: TenantId,
        id: PermissionOverrideId,
        revoked_by: Option<UserId>,
    ) -> Result<(), CoreError> {
        if self.overrides.get(tenant_id, id).await?.is_none() {
            return Err(CoreError::NotFound("permission_override"));
        }
        self.overrides.revoke(tenant_id, id, Utc::now()).await?;

        AuditService::new(self.audit.clone())
            .append(AppendAuditEntryCommand {
                tenant_id,
                actor_user_id: revoked_by,
                actor_type: "user".to_string(),
                action: "core.override.revoked".to_string(),
                resource_type: "permission_override".to_string(),
                resource_id: Some(id.as_uuid()),
                correlation_id: None,
                causation_id: None,
                payload: serde_json::json!({}),
                category: Some("authz".to_string()),
                ..Default::default()
            })
            .await?;

        self.outbox
            .publish(EventEnvelope::new(
                tenant_id,
                revoked_by
                    .map(|user_id| ActorRef::User { user_id })
                    .unwrap_or(ActorRef::System),
                ResourceRef {
                    resource_type: "permission_override".to_string(),
                    resource_id: id.as_uuid(),
                },
                None,
                None,
                CoreEvent::PermissionOverrideRevoked {
                    tenant_id,
                    override_id: id,
                },
            ))
            .await?;

        Ok(())
    }

    pub async fn list_permission_overrides(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<Vec<PermissionOverride>, CoreError> {
        self.overrides.list_for_user(tenant_id, user_id).await
    }

    /// Point lookup used by the thin `GET /api/v1/core/roles` catalog browse endpoint — not part
    /// of `AuthzApi` since it is not an authorization decision.
    pub async fn get_role(&self, id: RoleId) -> Result<Option<RoleDefinition>, CoreError> {
        self.roles.get(id).await
    }
}
