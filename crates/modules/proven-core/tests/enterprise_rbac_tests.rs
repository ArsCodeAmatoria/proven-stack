//! Enterprise RBAC integration tests (ADR-0007) — exercise `RoleEngine` + `PermissionEngine` +
//! policies through `CoreModule::in_memory()` via the public `AuthzApi` only, mirroring how
//! every other module is expected to consume Core (never reaching into `domain`/`infrastructure`
//! internals directly).

use chrono::{Duration, Utc};
use proven_core::application::services::{
    AuthorizeRequest, GrantAccessCommand, InviteUserCommand, ProvisionTenantCommand,
    ProvisionTenantResult, UpsertPermissionOverrideCommand,
};
use proven_core::domain::{permissions, AbacContext, AccessScope, CompanyType, GrantKind, OverrideEffect};
use proven_core::{AuthzApi, CoreModule, IdentityApi, TenancyApi};
use proven_shared::{PermissionCode, PrincipalId, ProjectId, RegionCode, UserId};
use uuid::Uuid;

async fn provision_test_tenant(module: &CoreModule) -> ProvisionTenantResult {
    module
        .services
        .provision_tenant(ProvisionTenantCommand {
            slug: format!("acme-{}", Uuid::new_v4()),
            display_name: "Acme Construction".into(),
            region_code: RegionCode::new("CA"),
            owner_company_name: "Acme GC Ltd".into(),
            owner_company_type: CompanyType::Prime,
            admin_email: format!("admin-{}@acme.test", Uuid::new_v4()),
            admin_display_name: "Acme Admin".into(),
            seats_limit: 25,
        })
        .await
        .expect("provision_tenant should succeed")
}

/// Invite + activate a fresh, grant-less user — the tenant admin's own Tenant-scope grant would
/// otherwise cover every resource and defeat scope-restriction assertions.
async fn fresh_active_user(module: &CoreModule, result: &ProvisionTenantResult) -> UserId {
    let user = module
        .services
        .invite_user(InviteUserCommand {
            tenant_id: result.tenant.id,
            email: format!("worker-{}@acme.test", Uuid::new_v4()),
            display_name: "Fresh Worker".into(),
            invited_by: Some(result.admin_user.id),
        })
        .await
        .expect("invite should succeed");
    module
        .services
        .activate_user(result.tenant.id, user.id)
        .await
        .expect("activate should succeed")
        .id
}

async fn authorize_as(
    module: &CoreModule,
    tenant_id: proven_shared::TenantId,
    user_id: UserId,
    permission: &str,
    resource: AccessScope,
) -> proven_core::AuthzDecision {
    module
        .services
        .authorize(AuthorizeRequest::new(
            tenant_id,
            PrincipalId::from_uuid(user_id.as_uuid()),
            PermissionCode::from(permission),
            resource,
        ))
        .await
        .expect("authorize should not error")
}

#[tokio::test]
async fn system_company_role_grants_company_scope() {
    let module = CoreModule::in_memory();
    let result = provision_test_tenant(&module).await;
    let worker = fresh_active_user(&module, &result).await;
    let company_id = result.owner_company.id.as_uuid();

    module
        .services
        .grant_access(GrantAccessCommand {
            tenant_id: result.tenant.id,
            user_id: worker,
            role_id: permissions::company_admin_role_id(),
            scope: AccessScope::company(company_id),
            grant_kind: GrantKind::Standard,
            expires_at: None,
            created_by: Some(result.admin_user.id),
        })
        .await
        .expect("grant_access should succeed");

    let allowed = authorize_as(
        &module,
        result.tenant.id,
        worker,
        permissions::COMPANY_MANAGE,
        AccessScope::company(company_id),
    )
    .await;
    assert!(
        allowed.is_allowed(),
        "Company Admin grant should cover its own company scope"
    );

    let other_company_denied = authorize_as(
        &module,
        result.tenant.id,
        worker,
        permissions::COMPANY_MANAGE,
        AccessScope::company(Uuid::new_v4()),
    )
    .await;
    assert!(
        !other_company_denied.is_allowed(),
        "Company Admin grant must not cover a different company"
    );
}

#[tokio::test]
async fn project_role_does_not_cover_other_project() {
    let module = CoreModule::in_memory();
    let result = provision_test_tenant(&module).await;
    let worker = fresh_active_user(&module, &result).await;
    let project_a = ProjectId::new();
    let project_b = ProjectId::new();

    module
        .services
        .grant_access(GrantAccessCommand {
            tenant_id: result.tenant.id,
            user_id: worker,
            role_id: permissions::project_admin_role_id(),
            scope: AccessScope::project(project_a.as_uuid()),
            grant_kind: GrantKind::Standard,
            expires_at: None,
            created_by: Some(result.admin_user.id),
        })
        .await
        .expect("grant_access should succeed");

    let allowed_on_a = authorize_as(
        &module,
        result.tenant.id,
        worker,
        permissions::MEMBERSHIP_MANAGE,
        AccessScope::project(project_a.as_uuid()),
    )
    .await;
    assert!(allowed_on_a.is_allowed(), "grant should cover project A");

    let denied_on_b = authorize_as(
        &module,
        result.tenant.id,
        worker,
        permissions::MEMBERSHIP_MANAGE,
        AccessScope::project(project_b.as_uuid()),
    )
    .await;
    assert!(
        !denied_on_b.is_allowed(),
        "Project-scoped grant must not cover a different project"
    );
}

#[tokio::test]
async fn temporary_grant_expires() {
    let module = CoreModule::in_memory();
    let result = provision_test_tenant(&module).await;
    let worker = fresh_active_user(&module, &result).await;

    module
        .services
        .grant_access(GrantAccessCommand {
            tenant_id: result.tenant.id,
            user_id: worker,
            role_id: permissions::temporary_elevated_role_id(),
            scope: AccessScope::tenant(),
            grant_kind: GrantKind::Temporary,
            // Already expired — RoleEngine::validate_expiry only requires expires_at to be
            // *present*, not in the future, so this grant is created successfully but inactive.
            expires_at: Some(Utc::now() - Duration::seconds(1)),
            created_by: Some(result.admin_user.id),
        })
        .await
        .expect("grant_access should succeed even with a past expiry");

    let decision = authorize_as(
        &module,
        result.tenant.id,
        worker,
        permissions::OVERRIDE_MANAGE,
        AccessScope::tenant(),
    )
    .await;
    assert!(
        !decision.is_allowed(),
        "an expired temporary grant must deny access (fail closed)"
    );
}

#[tokio::test]
async fn temporary_grant_without_expiry_is_rejected() {
    let module = CoreModule::in_memory();
    let result = provision_test_tenant(&module).await;
    let worker = fresh_active_user(&module, &result).await;

    let err = module
        .services
        .grant_access(GrantAccessCommand {
            tenant_id: result.tenant.id,
            user_id: worker,
            role_id: permissions::temporary_elevated_role_id(),
            scope: AccessScope::tenant(),
            grant_kind: GrantKind::Standard,
            expires_at: None,
            created_by: Some(result.admin_user.id),
        })
        .await;
    assert!(
        err.is_err(),
        "RoleEngine must reject a Temporary role grant without expires_at"
    );
}

#[tokio::test]
async fn deny_override_wins_over_role_grant() {
    let module = CoreModule::in_memory();
    let result = provision_test_tenant(&module).await;
    let worker = fresh_active_user(&module, &result).await;

    module
        .services
        .grant_access(GrantAccessCommand {
            tenant_id: result.tenant.id,
            user_id: worker,
            role_id: permissions::system_tenant_admin_role_id(),
            scope: AccessScope::tenant(),
            grant_kind: GrantKind::Standard,
            expires_at: None,
            created_by: Some(result.admin_user.id),
        })
        .await
        .expect("grant_access should succeed");

    let allowed_before_override = authorize_as(
        &module,
        result.tenant.id,
        worker,
        permissions::USER_MANAGE,
        AccessScope::tenant(),
    )
    .await;
    assert!(allowed_before_override.is_allowed());

    module
        .services
        .upsert_permission_override(UpsertPermissionOverrideCommand {
            tenant_id: result.tenant.id,
            user_id: worker,
            permission: PermissionCode::from(permissions::USER_MANAGE),
            effect: OverrideEffect::Deny,
            scope: AccessScope::tenant(),
            reason: Some("incident response".to_string()),
            expires_at: None,
            created_by: Some(result.admin_user.id),
        })
        .await
        .expect("upsert_permission_override should succeed");

    let denied_after_override = authorize_as(
        &module,
        result.tenant.id,
        worker,
        permissions::USER_MANAGE,
        AccessScope::tenant(),
    )
    .await;
    assert!(
        !denied_after_override.is_allowed(),
        "an active deny override must win over a covering role grant"
    );
}

#[tokio::test]
async fn allow_override_grants_without_role() {
    let module = CoreModule::in_memory();
    let result = provision_test_tenant(&module).await;
    let worker = fresh_active_user(&module, &result).await;

    let denied_without_grant = authorize_as(
        &module,
        result.tenant.id,
        worker,
        permissions::USER_MANAGE,
        AccessScope::tenant(),
    )
    .await;
    assert!(!denied_without_grant.is_allowed());

    module
        .services
        .upsert_permission_override(UpsertPermissionOverrideCommand {
            tenant_id: result.tenant.id,
            user_id: worker,
            permission: PermissionCode::from(permissions::USER_MANAGE),
            effect: OverrideEffect::Allow,
            scope: AccessScope::tenant(),
            reason: Some("emergency access".to_string()),
            expires_at: None,
            created_by: Some(result.admin_user.id),
        })
        .await
        .expect("upsert_permission_override should succeed");

    let allowed_via_override = authorize_as(
        &module,
        result.tenant.id,
        worker,
        permissions::USER_MANAGE,
        AccessScope::tenant(),
    )
    .await;
    assert!(
        allowed_via_override.is_allowed(),
        "an active allow override must grant access even without any role"
    );
}

#[tokio::test]
async fn sealed_resource_policy_denies_manage() {
    let module = CoreModule::in_memory();
    let result = provision_test_tenant(&module).await;

    // The tenant admin already holds a Tenant-scope Tenant Admin grant, so this would otherwise
    // be allowed — the sealed-resource policy must still deny it.
    let decision = module
        .services
        .authorize(AuthorizeRequest {
            tenant_id: result.tenant.id,
            principal: PrincipalId::from_uuid(result.admin_user.id.as_uuid()),
            permission: PermissionCode::from(permissions::COMPANY_MANAGE),
            resource: AccessScope::tenant(),
            abac: AbacContext {
                resource_state: Some("sealed".to_string()),
                ..AbacContext::empty()
            },
        })
        .await
        .expect("authorize should not error");
    assert!(
        !decision.is_allowed(),
        "sealed resources must deny manage-shaped permissions regardless of role"
    );

    // A read-shaped permission is unaffected by the sealed state.
    let read_decision = module
        .services
        .authorize(AuthorizeRequest {
            tenant_id: result.tenant.id,
            principal: PrincipalId::from_uuid(result.admin_user.id.as_uuid()),
            permission: PermissionCode::from(permissions::COMPANY_READ),
            resource: AccessScope::tenant(),
            abac: AbacContext {
                resource_state: Some("sealed".to_string()),
                ..AbacContext::empty()
            },
        })
        .await
        .expect("authorize should not error");
    assert!(read_decision.is_allowed());
}

#[tokio::test]
async fn module_disabled_denies_documents_permission() {
    let module = CoreModule::in_memory();
    let result = provision_test_tenant(&module).await;
    let worker = fresh_active_user(&module, &result).await;

    // Document Control holds documents.* permissions, but the trial license entitles only the
    // "core" module (see TenancyService::provision_tenant) — documents.* must deny closed.
    module
        .services
        .grant_access(GrantAccessCommand {
            tenant_id: result.tenant.id,
            user_id: worker,
            role_id: permissions::document_control_role_id(),
            scope: AccessScope::company(result.owner_company.id.as_uuid()),
            grant_kind: GrantKind::Standard,
            expires_at: None,
            created_by: Some(result.admin_user.id),
        })
        .await
        .expect("grant_access should succeed");

    // A second, unrelated grant with a `core.*` (never license-gated) permission at the same
    // scope — used below to prove the deny is specifically about module gating, not the scope
    // or the principal.
    module
        .services
        .grant_access(GrantAccessCommand {
            tenant_id: result.tenant.id,
            user_id: worker,
            role_id: permissions::company_admin_role_id(),
            scope: AccessScope::company(result.owner_company.id.as_uuid()),
            grant_kind: GrantKind::Standard,
            expires_at: None,
            created_by: Some(result.admin_user.id),
        })
        .await
        .expect("grant_access should succeed");

    let decision = authorize_as(
        &module,
        result.tenant.id,
        worker,
        permissions::DOCUMENTS_DOCUMENT_READ,
        AccessScope::company(result.owner_company.id.as_uuid()),
    )
    .await;
    assert!(
        !decision.is_allowed(),
        "documents.* must deny when the documents module is not licensed"
    );

    // Sanity check: a core.* permission on the very same grant/scope is unaffected by licensing.
    let core_permission_decision = authorize_as(
        &module,
        result.tenant.id,
        worker,
        permissions::COMPANY_READ,
        AccessScope::company(result.owner_company.id.as_uuid()),
    )
    .await;
    assert!(core_permission_decision.is_allowed());
}
