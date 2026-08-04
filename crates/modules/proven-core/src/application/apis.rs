//! In-process public interfaces (CORE_DOMAIN.md §13.1). Every other module and Temporal
//! activity talks to Core exclusively through these traits — never through Core's schema.

use std::sync::Arc;

use async_trait::async_trait;
use proven_shared::{
    CompanyId, FeatureFlagKey, FileObjectId, GrantId, ModuleKey, Page, PageRequest, PermissionCode,
    PermissionOverrideId, PrincipalId, ProjectId, SettingKey, TenantId, UserId,
};
use uuid::Uuid;

use crate::application::ports::{
    AuditRepository, CompanyRepository, EventPublisher, FileObjectRepository,
    FileShareLinkRepository, FlagsRepository, GrantRepository, LicenseRepository,
    ObjectStoragePort, OverrideRepository, ProjectMembershipRepository, RoleRepository,
    SettingsRepository, TeamRepository, TenantRepository, UserRepository, VirusScanHook,
};
use crate::application::services::{
    AppendAuditEntryCommand, ApplyScanResultCommand, AuditService, AuthorizeRequest, AuthzService,
    CreateFileUploadIntentCommand, CreatePublicShareLinkCommand, CreateTeamCommand, FileService,
    FlagsService, GrantAccessCommand, GrantProjectMembershipCommand, IdentityService,
    InviteUserCommand, LicenseService, MembershipService, ProvisionTenantCommand,
    ProvisionTenantResult, RegisterCompanyCommand, SettingsService, TenancyService,
    UpsertPermissionOverrideCommand, UpsertSettingCommand,
};
use crate::domain::{
    AccessGrant, AuditEntry, AuditExportJob, AuditRetentionPolicy, AuditSearchQuery, AuthzDecision,
    Company, CoreError, DownloadLink, FileObject, FileShareLink, License, PermissionOverride,
    ProjectMembership, SettingEntry, SettingScopeType, Team, Tenant, UploadIntent, User,
};
use crate::infrastructure::memory::MemoryStore;
use crate::infrastructure::object_storage::PlaceholderObjectStorage;
use crate::infrastructure::outbox::InMemoryOutbox;
use crate::infrastructure::virus_scan::PassthroughVirusScanHook;

#[async_trait]
pub trait TenancyApi: Send + Sync {
    async fn provision_tenant(
        &self,
        cmd: ProvisionTenantCommand,
    ) -> Result<ProvisionTenantResult, CoreError>;
    async fn get_tenant(&self, id: TenantId) -> Result<Tenant, CoreError>;
    async fn register_company(&self, cmd: RegisterCompanyCommand) -> Result<Company, CoreError>;
    async fn get_company(&self, id: CompanyId) -> Result<Company, CoreError>;
}

#[async_trait]
pub trait IdentityApi: Send + Sync {
    async fn invite_user(&self, cmd: InviteUserCommand) -> Result<User, CoreError>;
    async fn activate_user(&self, tenant_id: TenantId, user_id: UserId) -> Result<User, CoreError>;
    async fn get_user(&self, tenant_id: TenantId, user_id: UserId) -> Result<User, CoreError>;
}

#[async_trait]
pub trait AuthzApi: Send + Sync {
    /// The **only** permission decision path in the platform (ADR-0003).
    async fn authorize(&self, req: AuthorizeRequest) -> Result<AuthzDecision, CoreError>;
    async fn grant_access(&self, cmd: GrantAccessCommand) -> Result<AccessGrant, CoreError>;
    async fn revoke_access(
        &self,
        tenant_id: TenantId,
        grant_id: GrantId,
        revoked_by: Option<UserId>,
    ) -> Result<(), CoreError>;
    async fn list_effective_permissions(
        &self,
        tenant_id: TenantId,
        principal: PrincipalId,
    ) -> Result<Vec<PermissionCode>, CoreError>;

    /// Create an explicit allow/deny override for a principal (ADR-0007 §5). Deny overrides win
    /// over any covering role grant; allow overrides can grant access without any role at all
    /// (e.g. emergency/temporary access).
    async fn upsert_permission_override(
        &self,
        cmd: UpsertPermissionOverrideCommand,
    ) -> Result<PermissionOverride, CoreError>;

    async fn revoke_permission_override(
        &self,
        tenant_id: TenantId,
        id: PermissionOverrideId,
        revoked_by: Option<UserId>,
    ) -> Result<(), CoreError>;

    async fn list_permission_overrides(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<Vec<PermissionOverride>, CoreError>;
}

#[async_trait]
pub trait MembershipApi: Send + Sync {
    async fn grant_project_membership(
        &self,
        cmd: GrantProjectMembershipCommand,
    ) -> Result<ProjectMembership, CoreError>;
    async fn is_project_member(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        principal: PrincipalId,
    ) -> Result<bool, CoreError>;
    async fn list_principal_projects(
        &self,
        tenant_id: TenantId,
        principal: PrincipalId,
    ) -> Result<Vec<ProjectId>, CoreError>;
    async fn create_team(&self, cmd: CreateTeamCommand) -> Result<Team, CoreError>;
}

#[async_trait]
pub trait FileApi: Send + Sync {
    async fn create_upload_intent(
        &self,
        cmd: CreateFileUploadIntentCommand,
    ) -> Result<UploadIntent, CoreError>;

    async fn get_file(
        &self,
        tenant_id: TenantId,
        file_id: FileObjectId,
    ) -> Result<FileObject, CoreError>;

    async fn list_file_versions(
        &self,
        tenant_id: TenantId,
        file_id: FileObjectId,
    ) -> Result<Vec<FileObject>, CoreError>;

    async fn complete_upload(
        &self,
        tenant_id: TenantId,
        file_id: FileObjectId,
        checksum_sha256: String,
        byte_size: i64,
    ) -> Result<FileObject, CoreError>;

    async fn apply_scan_result(
        &self,
        cmd: ApplyScanResultCommand,
    ) -> Result<FileObject, CoreError>;

    async fn soft_delete_file(
        &self,
        tenant_id: TenantId,
        file_id: FileObjectId,
        deleted_by: Option<UserId>,
    ) -> Result<FileObject, CoreError>;

    async fn update_file_metadata(
        &self,
        tenant_id: TenantId,
        file_id: FileObjectId,
        metadata: serde_json::Value,
        updated_by: Option<UserId>,
    ) -> Result<FileObject, CoreError>;

    async fn create_private_download_link(
        &self,
        tenant_id: TenantId,
        file_id: FileObjectId,
        requested_by: Option<UserId>,
    ) -> Result<DownloadLink, CoreError>;

    async fn create_public_share_link(
        &self,
        cmd: CreatePublicShareLinkCommand,
    ) -> Result<FileShareLink, CoreError>;

    async fn resolve_public_share_link(&self, token: &str) -> Result<DownloadLink, CoreError>;

    async fn list_expired_temporaries(
        &self,
        tenant_id: TenantId,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<FileObject>, CoreError>;
}

/// Core's audit Source of Record surface (ADR-0008). `query` remains for back-compat callers;
/// `search`/`request_export`/retention methods are the Audit Engine's expanded API.
#[async_trait]
pub trait AuditApi: Send + Sync {
    async fn append(&self, cmd: AppendAuditEntryCommand) -> Result<AuditEntry, CoreError>;

    async fn query(
        &self,
        tenant_id: TenantId,
        page: PageRequest,
    ) -> Result<Page<AuditEntry>, CoreError>;

    async fn search(
        &self,
        tenant_id: TenantId,
        query: AuditSearchQuery,
        page: PageRequest,
    ) -> Result<Page<AuditEntry>, CoreError>;

    async fn request_export(
        &self,
        tenant_id: TenantId,
        requested_by: Option<UserId>,
        filter: AuditSearchQuery,
    ) -> Result<AuditExportJob, CoreError>;

    async fn get_export(
        &self,
        tenant_id: TenantId,
        job_id: Uuid,
    ) -> Result<AuditExportJob, CoreError>;

    async fn get_retention_policy(
        &self,
        tenant_id: TenantId,
    ) -> Result<AuditRetentionPolicy, CoreError>;

    async fn upsert_retention_policy(
        &self,
        policy: AuditRetentionPolicy,
    ) -> Result<AuditRetentionPolicy, CoreError>;
}

#[async_trait]
pub trait SettingsApi: Send + Sync {
    async fn get(
        &self,
        tenant_id: TenantId,
        scope_type: SettingScopeType,
        scope_id: Option<Uuid>,
        key: &SettingKey,
    ) -> Result<Option<SettingEntry>, CoreError>;
    async fn upsert(&self, cmd: UpsertSettingCommand) -> Result<SettingEntry, CoreError>;
}

#[async_trait]
pub trait FlagsApi: Send + Sync {
    async fn evaluate(
        &self,
        key: &FeatureFlagKey,
        tenant_id: Option<TenantId>,
        user_id: Option<UserId>,
    ) -> Result<bool, CoreError>;
}

#[async_trait]
pub trait LicenseApi: Send + Sync {
    async fn get_current(&self, tenant_id: TenantId) -> Result<License, CoreError>;
    async fn is_module_enabled(
        &self,
        tenant_id: TenantId,
        module: &ModuleKey,
    ) -> Result<bool, CoreError>;
}

/// Bundle of repository ports used to construct [`CoreServices`]. Swap `infrastructure::memory`
/// for `infrastructure::postgres` adapters without touching application logic (ADR-0004).
pub struct CorePorts {
    pub tenants: Arc<dyn TenantRepository>,
    pub companies: Arc<dyn CompanyRepository>,
    pub users: Arc<dyn UserRepository>,
    pub roles: Arc<dyn RoleRepository>,
    pub grants: Arc<dyn GrantRepository>,
    pub overrides: Arc<dyn OverrideRepository>,
    pub memberships: Arc<dyn ProjectMembershipRepository>,
    pub teams: Arc<dyn TeamRepository>,
    pub files: Arc<dyn FileObjectRepository>,
    pub file_links: Arc<dyn FileShareLinkRepository>,
    pub object_storage: Arc<dyn ObjectStoragePort>,
    pub virus_scan: Arc<dyn VirusScanHook>,
    pub audit: Arc<dyn AuditRepository>,
    pub settings: Arc<dyn SettingsRepository>,
    pub flags: Arc<dyn FlagsRepository>,
    pub license: Arc<dyn LicenseRepository>,
    pub outbox: Arc<dyn EventPublisher>,
}

impl CorePorts {
    /// Wire every port to a single shared, seeded in-memory store (unit tests / no-DB mode).
    pub fn in_memory() -> Self {
        let store = Arc::new(MemoryStore::seeded());
        let outbox = Arc::new(InMemoryOutbox::new());
        Self {
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
            object_storage: Arc::new(PlaceholderObjectStorage::new()),
            virus_scan: Arc::new(PassthroughVirusScanHook),
            audit: store.clone(),
            settings: store.clone(),
            flags: store.clone(),
            license: store,
            outbox,
        }
    }
}

/// Facade implementing every Core public interface — the seam other modules depend on.
pub struct CoreServices {
    tenancy: TenancyService,
    identity: IdentityService,
    authz: AuthzService,
    membership: MembershipService,
    files: FileService,
    audit: AuditService,
    settings: SettingsService,
    flags: FlagsService,
    license: LicenseService,
}

impl CoreServices {
    pub fn new(ports: CorePorts) -> Self {
        Self {
            tenancy: TenancyService::new(
                ports.tenants.clone(),
                ports.companies.clone(),
                ports.users.clone(),
                ports.roles.clone(),
                ports.grants.clone(),
                ports.license.clone(),
                ports.audit.clone(),
                ports.outbox.clone(),
            ),
            identity: IdentityService::new(
                ports.users.clone(),
                ports.audit.clone(),
                ports.outbox.clone(),
            ),
            authz: AuthzService::new(
                ports.tenants.clone(),
                ports.users.clone(),
                ports.roles.clone(),
                ports.grants.clone(),
                ports.overrides.clone(),
                ports.license.clone(),
                ports.audit.clone(),
                ports.outbox.clone(),
            ),
            membership: MembershipService::new(
                ports.memberships.clone(),
                ports.teams.clone(),
                ports.audit.clone(),
                ports.outbox.clone(),
            ),
            files: FileService::new(
                ports.files.clone(),
                ports.file_links.clone(),
                ports.object_storage.clone(),
                ports.virus_scan.clone(),
                ports.audit.clone(),
                ports.outbox.clone(),
            ),
            audit: AuditService::new(ports.audit.clone()).with_outbox(ports.outbox.clone()),
            settings: SettingsService::new(ports.settings.clone(), ports.outbox.clone()),
            flags: FlagsService::new(ports.flags.clone()),
            license: LicenseService::new(ports.license.clone()),
        }
    }

    /// Build `CoreServices` backed entirely by the seeded in-memory store (no Postgres needed).
    pub fn in_memory() -> Self {
        Self::new(CorePorts::in_memory())
    }

    /// Point lookup for the thin role-catalog browse endpoint (`GET /api/v1/core/roles`) — not
    /// part of `AuthzApi` since it is not an authorization decision.
    pub async fn get_role(
        &self,
        id: proven_shared::RoleId,
    ) -> Result<Option<crate::domain::RoleDefinition>, CoreError> {
        self.authz.get_role(id).await
    }

    /// Ids eligible for archival under the tenant's retention policy — **never deletes**
    /// anything (ADR-0008 append-only hard rule). Not part of `AuditApi` since it is an
    /// ops/back-office concern, not a general-purpose module capability.
    pub async fn list_audit_purge_candidates(
        &self,
        tenant_id: TenantId,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<proven_shared::AuditEntryId>, CoreError> {
        self.audit.list_purge_candidates(tenant_id, now).await
    }
}

#[async_trait]
impl TenancyApi for CoreServices {
    async fn provision_tenant(
        &self,
        cmd: ProvisionTenantCommand,
    ) -> Result<ProvisionTenantResult, CoreError> {
        self.tenancy.provision_tenant(cmd).await
    }

    async fn get_tenant(&self, id: TenantId) -> Result<Tenant, CoreError> {
        self.tenancy.get_tenant(id).await
    }

    async fn register_company(&self, cmd: RegisterCompanyCommand) -> Result<Company, CoreError> {
        self.tenancy.register_company(cmd).await
    }

    async fn get_company(&self, id: CompanyId) -> Result<Company, CoreError> {
        self.tenancy.get_company(id).await
    }
}

#[async_trait]
impl IdentityApi for CoreServices {
    async fn invite_user(&self, cmd: InviteUserCommand) -> Result<User, CoreError> {
        self.identity.invite_user(cmd).await
    }

    async fn activate_user(&self, tenant_id: TenantId, user_id: UserId) -> Result<User, CoreError> {
        self.identity.activate_user(tenant_id, user_id).await
    }

    async fn get_user(&self, tenant_id: TenantId, user_id: UserId) -> Result<User, CoreError> {
        self.identity.get_user(tenant_id, user_id).await
    }
}

#[async_trait]
impl AuthzApi for CoreServices {
    async fn authorize(&self, req: AuthorizeRequest) -> Result<AuthzDecision, CoreError> {
        self.authz.authorize(req).await
    }

    async fn grant_access(&self, cmd: GrantAccessCommand) -> Result<AccessGrant, CoreError> {
        self.authz.grant_access(cmd).await
    }

    async fn revoke_access(
        &self,
        tenant_id: TenantId,
        grant_id: GrantId,
        revoked_by: Option<UserId>,
    ) -> Result<(), CoreError> {
        self.authz
            .revoke_access(tenant_id, grant_id, revoked_by)
            .await
    }

    async fn list_effective_permissions(
        &self,
        tenant_id: TenantId,
        principal: PrincipalId,
    ) -> Result<Vec<PermissionCode>, CoreError> {
        self.authz
            .list_effective_permissions(tenant_id, principal)
            .await
    }

    async fn upsert_permission_override(
        &self,
        cmd: UpsertPermissionOverrideCommand,
    ) -> Result<PermissionOverride, CoreError> {
        self.authz.upsert_permission_override(cmd).await
    }

    async fn revoke_permission_override(
        &self,
        tenant_id: TenantId,
        id: PermissionOverrideId,
        revoked_by: Option<UserId>,
    ) -> Result<(), CoreError> {
        self.authz
            .revoke_permission_override(tenant_id, id, revoked_by)
            .await
    }

    async fn list_permission_overrides(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<Vec<PermissionOverride>, CoreError> {
        self.authz.list_permission_overrides(tenant_id, user_id).await
    }
}

#[async_trait]
impl MembershipApi for CoreServices {
    async fn grant_project_membership(
        &self,
        cmd: GrantProjectMembershipCommand,
    ) -> Result<ProjectMembership, CoreError> {
        self.membership.grant_project_membership(cmd).await
    }

    async fn is_project_member(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        principal: PrincipalId,
    ) -> Result<bool, CoreError> {
        self.membership
            .is_project_member(tenant_id, project_id, principal)
            .await
    }

    async fn list_principal_projects(
        &self,
        tenant_id: TenantId,
        principal: PrincipalId,
    ) -> Result<Vec<ProjectId>, CoreError> {
        self.membership
            .list_principal_projects(tenant_id, principal)
            .await
    }

    async fn create_team(&self, cmd: CreateTeamCommand) -> Result<Team, CoreError> {
        self.membership.create_team(cmd).await
    }
}

#[async_trait]
impl FileApi for CoreServices {
    async fn create_upload_intent(
        &self,
        cmd: CreateFileUploadIntentCommand,
    ) -> Result<UploadIntent, CoreError> {
        self.files.create_upload_intent(cmd).await
    }

    async fn get_file(
        &self,
        tenant_id: TenantId,
        file_id: FileObjectId,
    ) -> Result<FileObject, CoreError> {
        self.files.get_file(tenant_id, file_id).await
    }

    async fn list_file_versions(
        &self,
        tenant_id: TenantId,
        file_id: FileObjectId,
    ) -> Result<Vec<FileObject>, CoreError> {
        self.files.list_versions(tenant_id, file_id).await
    }

    async fn complete_upload(
        &self,
        tenant_id: TenantId,
        file_id: FileObjectId,
        checksum_sha256: String,
        byte_size: i64,
    ) -> Result<FileObject, CoreError> {
        self.files
            .complete_upload(tenant_id, file_id, checksum_sha256, byte_size)
            .await
    }

    async fn apply_scan_result(
        &self,
        cmd: ApplyScanResultCommand,
    ) -> Result<FileObject, CoreError> {
        self.files.apply_scan_result(cmd).await
    }

    async fn soft_delete_file(
        &self,
        tenant_id: TenantId,
        file_id: FileObjectId,
        deleted_by: Option<UserId>,
    ) -> Result<FileObject, CoreError> {
        self.files.soft_delete(tenant_id, file_id, deleted_by).await
    }

    async fn update_file_metadata(
        &self,
        tenant_id: TenantId,
        file_id: FileObjectId,
        metadata: serde_json::Value,
        updated_by: Option<UserId>,
    ) -> Result<FileObject, CoreError> {
        self.files
            .update_metadata(tenant_id, file_id, metadata, updated_by)
            .await
    }

    async fn create_private_download_link(
        &self,
        tenant_id: TenantId,
        file_id: FileObjectId,
        requested_by: Option<UserId>,
    ) -> Result<DownloadLink, CoreError> {
        self.files
            .create_private_download_link(tenant_id, file_id, requested_by)
            .await
    }

    async fn create_public_share_link(
        &self,
        cmd: CreatePublicShareLinkCommand,
    ) -> Result<FileShareLink, CoreError> {
        self.files.create_public_share_link(cmd).await
    }

    async fn resolve_public_share_link(&self, token: &str) -> Result<DownloadLink, CoreError> {
        self.files.resolve_public_share_link(token).await
    }

    async fn list_expired_temporaries(
        &self,
        tenant_id: TenantId,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<FileObject>, CoreError> {
        self.files.list_expired_temporaries(tenant_id, now).await
    }
}

#[async_trait]
impl AuditApi for CoreServices {
    async fn append(&self, cmd: AppendAuditEntryCommand) -> Result<AuditEntry, CoreError> {
        self.audit.append(cmd).await
    }

    async fn query(
        &self,
        tenant_id: TenantId,
        page: PageRequest,
    ) -> Result<Page<AuditEntry>, CoreError> {
        self.audit.query(tenant_id, page).await
    }

    async fn search(
        &self,
        tenant_id: TenantId,
        query: AuditSearchQuery,
        page: PageRequest,
    ) -> Result<Page<AuditEntry>, CoreError> {
        self.audit.search(tenant_id, query, page).await
    }

    async fn request_export(
        &self,
        tenant_id: TenantId,
        requested_by: Option<UserId>,
        filter: AuditSearchQuery,
    ) -> Result<AuditExportJob, CoreError> {
        self.audit.request_export(tenant_id, requested_by, filter).await
    }

    async fn get_export(
        &self,
        tenant_id: TenantId,
        job_id: Uuid,
    ) -> Result<AuditExportJob, CoreError> {
        self.audit.get_export(tenant_id, job_id).await
    }

    async fn get_retention_policy(
        &self,
        tenant_id: TenantId,
    ) -> Result<AuditRetentionPolicy, CoreError> {
        self.audit.get_retention_policy(tenant_id).await
    }

    async fn upsert_retention_policy(
        &self,
        policy: AuditRetentionPolicy,
    ) -> Result<AuditRetentionPolicy, CoreError> {
        self.audit.upsert_retention_policy(policy).await
    }
}

#[async_trait]
impl SettingsApi for CoreServices {
    async fn get(
        &self,
        tenant_id: TenantId,
        scope_type: SettingScopeType,
        scope_id: Option<Uuid>,
        key: &SettingKey,
    ) -> Result<Option<SettingEntry>, CoreError> {
        self.settings
            .get(tenant_id, scope_type, scope_id, key)
            .await
    }

    async fn upsert(&self, cmd: UpsertSettingCommand) -> Result<SettingEntry, CoreError> {
        self.settings.upsert(cmd).await
    }
}

#[async_trait]
impl FlagsApi for CoreServices {
    async fn evaluate(
        &self,
        key: &FeatureFlagKey,
        tenant_id: Option<TenantId>,
        user_id: Option<UserId>,
    ) -> Result<bool, CoreError> {
        self.flags.evaluate(key, tenant_id, user_id).await
    }
}

#[async_trait]
impl LicenseApi for CoreServices {
    async fn get_current(&self, tenant_id: TenantId) -> Result<License, CoreError> {
        self.license.get_current(tenant_id).await
    }

    async fn is_module_enabled(
        &self,
        tenant_id: TenantId,
        module: &ModuleKey,
    ) -> Result<bool, CoreError> {
        self.license.is_module_enabled(tenant_id, module).await
    }
}
