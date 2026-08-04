//! Integration-style tests exercising `CoreModule` through the public `*Api` traits only —
//! mirrors how other modules are expected to consume Core.

use std::sync::Arc;

use chrono::Utc;
use proven_core::application::ports::FlagsRepository;
use proven_core::application::services::{
    AppendAuditEntryCommand, AuthorizeRequest, GrantAccessCommand, GrantProjectMembershipCommand,
    InviteUserCommand, ProvisionTenantCommand, ProvisionTenantResult, UpsertSettingCommand,
};
use proven_core::domain::{permissions, CompanyType, FeatureFlag, GrantKind, SettingScopeType};
use proven_core::infrastructure::{InMemoryOutbox, MemoryStore};
use proven_core::{
    AccessScope, AuditApi, AuthzApi, CoreModule, CorePorts, CoreServices, FlagsApi, IdentityApi,
    LicenseApi, MembershipApi, SettingsApi, TenancyApi,
};
use proven_shared::{
    FeatureFlagKey, ModuleKey, PageRequest, PermissionCode, PrincipalId, ProjectId, RegionCode,
    SettingKey,
};

async fn provision_test_tenant(module: &CoreModule) -> ProvisionTenantResult {
    module
        .services
        .provision_tenant(ProvisionTenantCommand {
            slug: "acme-construction".into(),
            display_name: "Acme Construction".into(),
            region_code: RegionCode::new("CA"),
            owner_company_name: "Acme GC Ltd".into(),
            owner_company_type: CompanyType::Prime,
            admin_email: "admin@acme.test".into(),
            admin_display_name: "Acme Admin".into(),
            seats_limit: 25,
        })
        .await
        .expect("provision_tenant should succeed")
}

#[tokio::test]
async fn provision_tenant_creates_admin_and_license() {
    let module = CoreModule::in_memory();
    let result = provision_test_tenant(&module).await;

    assert_eq!(result.tenant.slug, "acme-construction");
    assert_eq!(result.admin_user.email, "admin@acme.test");
    assert_eq!(result.owner_company.tenant_id, result.tenant.id);

    let enabled = module
        .services
        .is_module_enabled(result.tenant.id, &ModuleKey("core".to_string()))
        .await
        .expect("license check should succeed");
    assert!(
        enabled,
        "core module should be enabled on the trial license"
    );

    let fetched_tenant = module
        .services
        .get_tenant(result.tenant.id)
        .await
        .expect("tenant should be fetchable");
    assert_eq!(fetched_tenant.id, result.tenant.id);
}

#[tokio::test]
async fn authorize_allows_with_grant_denies_without() {
    let module = CoreModule::in_memory();
    let result = provision_test_tenant(&module).await;

    // The provisioned admin already holds a Tenant-scope Tenant Admin grant.
    let allow_decision = module
        .services
        .authorize(AuthorizeRequest {
            tenant_id: result.tenant.id,
            principal: PrincipalId::from_uuid(result.admin_user.id.as_uuid()),
            permission: PermissionCode::from(permissions::USER_MANAGE),
            resource: AccessScope::tenant(),
            abac: proven_core::domain::AbacContext::empty(),
        })
        .await
        .expect("authorize should not error");
    assert!(
        allow_decision.is_allowed(),
        "tenant admin should be allowed"
    );

    // A brand-new user with no grants must be denied (fail closed).
    let other_user = module
        .services
        .invite_user(InviteUserCommand {
            tenant_id: result.tenant.id,
            email: "worker@acme.test".into(),
            display_name: "Worker".into(),
            invited_by: Some(result.admin_user.id),
        })
        .await
        .expect("invite should succeed");
    let activated = module
        .services
        .activate_user(result.tenant.id, other_user.id)
        .await
        .expect("activate should succeed");

    let deny_decision = module
        .services
        .authorize(AuthorizeRequest {
            tenant_id: result.tenant.id,
            principal: PrincipalId::from_uuid(activated.id.as_uuid()),
            permission: PermissionCode::from(permissions::USER_MANAGE),
            resource: AccessScope::tenant(),
            abac: proven_core::domain::AbacContext::empty(),
        })
        .await
        .expect("authorize should not error");
    assert!(
        !deny_decision.is_allowed(),
        "user without a covering grant must be denied"
    );

    // Granting the Tenant Admin role to the worker should flip the decision to Allow.
    module
        .services
        .grant_access(GrantAccessCommand {
            tenant_id: result.tenant.id,
            user_id: activated.id,
            role_id: permissions::system_tenant_admin_role_id(),
            scope: AccessScope::tenant(),
            grant_kind: GrantKind::Standard,
            expires_at: None,
            created_by: Some(result.admin_user.id),
        })
        .await
        .expect("grant_access should succeed");

    let allow_after_grant = module
        .services
        .authorize(AuthorizeRequest {
            tenant_id: result.tenant.id,
            principal: PrincipalId::from_uuid(activated.id.as_uuid()),
            permission: PermissionCode::from(permissions::USER_MANAGE),
            resource: AccessScope::tenant(),
            abac: proven_core::domain::AbacContext::empty(),
        })
        .await
        .expect("authorize should not error");
    assert!(allow_after_grant.is_allowed());
}

#[tokio::test]
async fn project_membership_round_trip() {
    let module = CoreModule::in_memory();
    let result = provision_test_tenant(&module).await;
    let project_id = ProjectId::new();
    let principal = PrincipalId::from_uuid(result.admin_user.id.as_uuid());

    let not_yet_member = module
        .services
        .is_project_member(result.tenant.id, project_id, principal)
        .await
        .expect("query should not error");
    assert!(!not_yet_member);

    module
        .services
        .grant_project_membership(GrantProjectMembershipCommand {
            tenant_id: result.tenant.id,
            project_id,
            user_id: Some(result.admin_user.id),
            person_id: None,
            membership_role: "supervisor".into(),
            granted_by: Some(result.admin_user.id),
        })
        .await
        .expect("grant_project_membership should succeed");

    let is_member = module
        .services
        .is_project_member(result.tenant.id, project_id, principal)
        .await
        .expect("query should not error");
    assert!(is_member);

    let projects = module
        .services
        .list_principal_projects(result.tenant.id, principal)
        .await
        .expect("list should not error");
    assert_eq!(projects, vec![project_id]);
}

#[tokio::test]
async fn audit_append_is_immutable_digest() {
    let module = CoreModule::in_memory();
    let result = provision_test_tenant(&module).await;

    let payload = serde_json::json!({ "hello": "world" });
    let entry = module
        .services
        .append(AppendAuditEntryCommand {
            tenant_id: result.tenant.id,
            actor_user_id: Some(result.admin_user.id),
            actor_type: "user".into(),
            action: "test.action".into(),
            resource_type: "test_resource".into(),
            resource_id: None,
            correlation_id: None,
            causation_id: None,
            payload: payload.clone(),
            ..Default::default()
        })
        .await
        .expect("append should succeed");

    let expected_digest =
        proven_core::application::services::audit_service::digest_payload(&payload)
            .expect("digest should compute");
    assert_eq!(entry.payload_digest, expected_digest);

    // Re-appending identical content produces a new, distinct entry id (append-only log) but
    // an identical digest, since the digest reflects payload integrity, not entry uniqueness.
    let entry2 = module
        .services
        .append(AppendAuditEntryCommand {
            tenant_id: result.tenant.id,
            actor_user_id: Some(result.admin_user.id),
            actor_type: "user".into(),
            action: "test.action".into(),
            resource_type: "test_resource".into(),
            resource_id: None,
            correlation_id: None,
            causation_id: None,
            payload,
            ..Default::default()
        })
        .await
        .expect("append should succeed");
    assert_ne!(entry.id, entry2.id);
    assert_eq!(entry.payload_digest, entry2.payload_digest);

    let page = module
        .services
        .query(result.tenant.id, PageRequest::default())
        .await
        .expect("query should succeed");
    assert!(page.items.len() >= 2, "provisioning also appends an entry");
}

#[tokio::test]
async fn flag_override_wins() {
    // Built manually (rather than via `CoreModule::in_memory()`) so the test can reach the
    // `FlagsRepository` port directly to seed a flag definition + overrides.
    let store = Arc::new(MemoryStore::seeded());
    let outbox = Arc::new(InMemoryOutbox::new());
    let services = CoreServices::new(CorePorts {
        tenants: store.clone(),
        companies: store.clone(),
        users: store.clone(),
        roles: store.clone(),
        grants: store.clone(),
        overrides: store.clone(),
        memberships: store.clone(),
        teams: store.clone(),
        files: store.clone(),
        file_links: store.clone(),
        object_storage: Arc::new(proven_core::PlaceholderObjectStorage::new()),
        virus_scan: Arc::new(proven_core::PassthroughVirusScanHook),
        audit: store.clone(),
        settings: store.clone(),
        flags: store.clone(),
        license: store.clone(),
        outbox,
    });
    let module = CoreModule {
        services: Arc::new(services),
    };
    let result = provision_test_tenant(&module).await;

    let key = FeatureFlagKey("core.test_flag".to_string());
    store
        .define_flag(&FeatureFlag {
            key: key.clone(),
            description: "test flag".into(),
            default_enabled: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
        .await
        .expect("define_flag should succeed");

    let default_eval = module
        .services
        .evaluate(&key, Some(result.tenant.id), None)
        .await
        .expect("evaluate should not error");
    assert!(!default_eval, "no override present — default should apply");

    store
        .set_override(&key, Some(result.tenant.id), None, true)
        .await
        .expect("tenant override should be set");

    let tenant_override_eval = module
        .services
        .evaluate(&key, Some(result.tenant.id), None)
        .await
        .expect("evaluate should not error");
    assert!(
        tenant_override_eval,
        "tenant override should win over default"
    );

    store
        .set_override(
            &key,
            Some(result.tenant.id),
            Some(result.admin_user.id),
            false,
        )
        .await
        .expect("user override should be set");

    let user_override_eval = module
        .services
        .evaluate(&key, Some(result.tenant.id), Some(result.admin_user.id))
        .await
        .expect("evaluate should not error");
    assert!(
        !user_override_eval,
        "user override should win over both tenant override and default"
    );

    // Sanity check that SettingsApi is wired correctly end to end.
    let entry = module
        .services
        .upsert(UpsertSettingCommand {
            tenant_id: result.tenant.id,
            scope_type: SettingScopeType::Tenant,
            scope_id: None,
            key: SettingKey("core.test.setting".to_string()),
            value: serde_json::json!({ "enabled": true }),
        })
        .await
        .expect("settings upsert should succeed");
    assert_eq!(entry.value, serde_json::json!({ "enabled": true }));
}
