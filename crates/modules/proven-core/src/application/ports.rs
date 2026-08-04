//! Repository / outbound ports. Implemented by `infrastructure::memory` (always) and
//! `infrastructure::postgres` (where feasible). Application services depend only on these
//! traits — never on a concrete storage engine (ADR-0004).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use proven_shared::{
    CompanyId, FeatureFlagKey, FileObjectId, GrantId, LicenseId, Page, PageRequest,
    PermissionOverrideId, ProjectId, RoleId, SettingKey, TeamId, TenantId, UserId,
};

use crate::domain::{
    AccessGrant, AuditEntry, AuditExportJob, AuditRetentionPolicy, AuditSearchQuery, Company,
    CoreError, FeatureFlag, FileObject, License, ModuleEntitlement, PermissionOverride,
    ProjectMembership, RoleDefinition, SettingEntry, SettingScopeType, Team, TeamMember, Tenant,
    User,
};
use crate::events::EventEnvelope;

#[async_trait]
pub trait TenantRepository: Send + Sync {
    async fn insert(&self, tenant: &Tenant) -> Result<(), CoreError>;
    async fn get(&self, id: TenantId) -> Result<Option<Tenant>, CoreError>;
    async fn update(&self, tenant: &Tenant) -> Result<(), CoreError>;
}

#[async_trait]
pub trait CompanyRepository: Send + Sync {
    async fn insert(&self, company: &Company) -> Result<(), CoreError>;
    async fn get(&self, id: CompanyId) -> Result<Option<Company>, CoreError>;
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn insert(&self, user: &User) -> Result<(), CoreError>;
    async fn get(&self, tenant_id: TenantId, id: UserId) -> Result<Option<User>, CoreError>;
    async fn get_by_email(
        &self,
        tenant_id: TenantId,
        email: &str,
    ) -> Result<Option<User>, CoreError>;
    async fn update(&self, user: &User) -> Result<(), CoreError>;
}

#[async_trait]
pub trait RoleRepository: Send + Sync {
    async fn insert(&self, role: &RoleDefinition) -> Result<(), CoreError>;
    async fn get(&self, id: RoleId) -> Result<Option<RoleDefinition>, CoreError>;
}

#[async_trait]
pub trait GrantRepository: Send + Sync {
    async fn insert(&self, grant: &AccessGrant) -> Result<(), CoreError>;
    async fn get(&self, tenant_id: TenantId, id: GrantId)
        -> Result<Option<AccessGrant>, CoreError>;
    async fn revoke(
        &self,
        tenant_id: TenantId,
        id: GrantId,
        revoked_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), CoreError>;
    async fn list_for_user(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<Vec<AccessGrant>, CoreError>;
}

#[async_trait]
pub trait ProjectMembershipRepository: Send + Sync {
    async fn insert(&self, membership: &ProjectMembership) -> Result<(), CoreError>;
    async fn find_active(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        user_id: UserId,
    ) -> Result<Option<ProjectMembership>, CoreError>;
    async fn list_for_user(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<Vec<ProjectMembership>, CoreError>;
}

#[async_trait]
pub trait TeamRepository: Send + Sync {
    async fn insert(&self, team: &Team) -> Result<(), CoreError>;
    async fn get(&self, tenant_id: TenantId, id: TeamId) -> Result<Option<Team>, CoreError>;
    async fn add_member(&self, member: &TeamMember) -> Result<(), CoreError>;
}

#[async_trait]
pub trait FileObjectRepository: Send + Sync {
    async fn insert(&self, file: &FileObject) -> Result<(), CoreError>;
    async fn get(
        &self,
        tenant_id: TenantId,
        id: FileObjectId,
    ) -> Result<Option<FileObject>, CoreError>;
    async fn update(&self, file: &FileObject) -> Result<(), CoreError>;
    async fn list_versions(
        &self,
        tenant_id: TenantId,
        root_file_id: FileObjectId,
    ) -> Result<Vec<FileObject>, CoreError>;
    /// Temporary uploads whose `expires_at` is at or before `now` (sweeper candidates).
    async fn list_expired_temporaries(
        &self,
        tenant_id: TenantId,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<FileObject>, CoreError>;
}

#[async_trait]
pub trait FileShareLinkRepository: Send + Sync {
    async fn insert(&self, link: &crate::domain::FileShareLink) -> Result<(), CoreError>;
    async fn get_by_token(
        &self,
        token: &str,
    ) -> Result<Option<crate::domain::FileShareLink>, CoreError>;
    async fn update(&self, link: &crate::domain::FileShareLink) -> Result<(), CoreError>;
}

/// Cloudflare R2 (S3-compatible) object storage port — bytes only (ADR-0010).
#[async_trait]
pub trait ObjectStoragePort: Send + Sync {
    async fn presign_put(
        &self,
        key: &str,
        content_type: &str,
        ttl_secs: u64,
    ) -> Result<crate::domain::PresignedUrl, CoreError>;

    async fn presign_get(
        &self,
        key: &str,
        ttl_secs: u64,
        filename: Option<&str>,
    ) -> Result<crate::domain::PresignedUrl, CoreError>;

    async fn delete_object(&self, key: &str) -> Result<(), CoreError>;
}

/// Virus / malware scan hook invoked after upload complete (ADR-0010).
/// Production wires enqueue-to-media-worker; default is pass-through Clean for tests.
#[async_trait]
pub trait VirusScanHook: Send + Sync {
    async fn scan(
        &self,
        req: crate::domain::VirusScanRequest,
    ) -> Result<crate::domain::VirusScanOutcome, CoreError>;
}

/// Core's append-only audit Source of Record port (ADR-0008). `query` is a back-compat thin
/// wrapper over `search` with an empty filter — new callers should prefer `search`.
#[async_trait]
pub trait AuditRepository: Send + Sync {
    async fn append(&self, entry: &AuditEntry) -> Result<(), CoreError>;

    async fn query(
        &self,
        tenant_id: TenantId,
        page: PageRequest,
    ) -> Result<Page<AuditEntry>, CoreError>;

    async fn search(
        &self,
        tenant_id: TenantId,
        query: &AuditSearchQuery,
        page: PageRequest,
    ) -> Result<Page<AuditEntry>, CoreError>;

    /// Most recently appended entry's `integrity_hash` for this tenant, if any — used to chain
    /// the next entry's `integrity_prev_hash` (ADR-0008 §3).
    async fn last_integrity_hash(&self, tenant_id: TenantId) -> Result<Option<String>, CoreError>;

    async fn insert_export_job(&self, job: &AuditExportJob) -> Result<(), CoreError>;
    async fn update_export_job(&self, job: &AuditExportJob) -> Result<(), CoreError>;
    async fn get_export_job(
        &self,
        tenant_id: TenantId,
        job_id: Uuid,
    ) -> Result<Option<AuditExportJob>, CoreError>;

    async fn get_retention_policy(
        &self,
        tenant_id: TenantId,
    ) -> Result<Option<AuditRetentionPolicy>, CoreError>;
    async fn upsert_retention_policy(&self, policy: &AuditRetentionPolicy)
        -> Result<(), CoreError>;
}

#[async_trait]
pub trait SettingsRepository: Send + Sync {
    async fn get(
        &self,
        tenant_id: TenantId,
        scope_type: SettingScopeType,
        scope_id: Option<Uuid>,
        key: &SettingKey,
    ) -> Result<Option<SettingEntry>, CoreError>;
    async fn upsert(&self, entry: &SettingEntry) -> Result<(), CoreError>;
}

#[async_trait]
pub trait FlagsRepository: Send + Sync {
    async fn get_flag(&self, key: &FeatureFlagKey) -> Result<Option<FeatureFlag>, CoreError>;
    async fn define_flag(&self, flag: &FeatureFlag) -> Result<(), CoreError>;
    async fn get_override(
        &self,
        key: &FeatureFlagKey,
        tenant_id: Option<TenantId>,
        user_id: Option<UserId>,
    ) -> Result<Option<bool>, CoreError>;
    async fn set_override(
        &self,
        key: &FeatureFlagKey,
        tenant_id: Option<TenantId>,
        user_id: Option<UserId>,
        enabled: bool,
    ) -> Result<(), CoreError>;
}

#[async_trait]
pub trait LicenseRepository: Send + Sync {
    async fn insert(
        &self,
        license: &License,
        entitlements: &[ModuleEntitlement],
    ) -> Result<(), CoreError>;
    async fn get_current(&self, tenant_id: TenantId) -> Result<Option<License>, CoreError>;
    async fn get_entitlements(
        &self,
        license_id: LicenseId,
    ) -> Result<Vec<ModuleEntitlement>, CoreError>;
}

/// Explicit allow/deny overrides evaluated by `PermissionEngine` after grants — ADR-0007 §5.
#[async_trait]
pub trait OverrideRepository: Send + Sync {
    async fn insert(&self, override_: &PermissionOverride) -> Result<(), CoreError>;
    async fn get(
        &self,
        tenant_id: TenantId,
        id: PermissionOverrideId,
    ) -> Result<Option<PermissionOverride>, CoreError>;
    async fn list_for_user(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<Vec<PermissionOverride>, CoreError>;
    async fn revoke(
        &self,
        tenant_id: TenantId,
        id: PermissionOverrideId,
        revoked_at: DateTime<Utc>,
    ) -> Result<(), CoreError>;
}

/// Outbound event transport (in-memory buffer for tests; NATS/outbox in production —
/// see ADR-0004: Core events ride `platform.outbox_messages`).
#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, envelope: EventEnvelope) -> Result<(), CoreError>;
}
