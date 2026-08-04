//! Full in-memory store implementing every repository port. Used for unit tests and any
//! no-Postgres deployment mode (ADR-0004 requires unit tests to run against an in-memory repo).

use std::collections::HashMap;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use proven_shared::{
    CompanyId, FeatureFlagKey, FileObjectId, GrantId, LicenseId, Page, PageRequest,
    PermissionOverrideId, ProjectId, RoleId, SettingKey, TeamId, TenantId, UserId,
};
use uuid::Uuid;

use crate::application::ports::{
    AuditRepository, CompanyRepository, FileObjectRepository, FlagsRepository, GrantRepository,
    LicenseRepository, OverrideRepository, ProjectMembershipRepository, RoleRepository,
    SettingsRepository, TeamRepository, TenantRepository, UserRepository,
};
use crate::domain::permissions::{
    self, system_tenant_admin_role_id, ALL_CORE_PERMISSIONS, APPROVALS_REQUEST_APPROVE,
    EQUIPMENT_PERMISSIONS, FEATURE_MODULE_ACCESS, PROJECTS_PROJECT_READ, SAFETY_PERMISSIONS,
    TRAINING_COMPLETION_RECORD, TRAINING_PERMISSIONS,
};
use crate::domain::{
    AccessGrant, AuditEntry, AuditExportJob, AuditRetentionPolicy, AuditSearchQuery, Company,
    CoreError, FeatureFlag, FileObject, FileObjectStatus, License, MembershipStatus,
    ModuleEntitlement, PermissionOverride, ProjectMembership, RoleDefinition, RoleKind, RoleStatus,
    SettingEntry, SettingScopeType, Team, TeamMember, Tenant, User,
};

#[derive(Default)]
struct MemoryState {
    tenants: HashMap<Uuid, Tenant>,
    companies: HashMap<Uuid, Company>,
    users: HashMap<Uuid, User>,
    roles: HashMap<Uuid, RoleDefinition>,
    grants: HashMap<Uuid, AccessGrant>,
    overrides: HashMap<Uuid, PermissionOverride>,
    memberships: HashMap<Uuid, ProjectMembership>,
    teams: HashMap<Uuid, Team>,
    team_members: Vec<TeamMember>,
    files: HashMap<Uuid, FileObject>,
    file_share_links: HashMap<String, crate::domain::FileShareLink>,
    audit: Vec<AuditEntry>,
    audit_export_jobs: HashMap<Uuid, AuditExportJob>,
    audit_retention_policies: HashMap<Uuid, AuditRetentionPolicy>,
    settings: Vec<SettingEntry>,
    flags: HashMap<String, FeatureFlag>,
    flag_overrides: Vec<FlagOverride>,
    /// One current license per tenant (matches the SQL partial-unique-index invariant).
    licenses: HashMap<Uuid, License>,
    entitlements: HashMap<Uuid, Vec<ModuleEntitlement>>,
}

struct FlagOverride {
    key: String,
    tenant_id: Option<Uuid>,
    user_id: Option<Uuid>,
    enabled: bool,
}

/// Shared, thread-safe in-memory backing store for every Core port.
pub struct MemoryStore {
    state: RwLock<MemoryState>,
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(MemoryState::default()),
        }
    }

    /// Seed the system Tenant Admin role + `core.*` permission catalog, plus the ADR-0007
    /// enterprise RBAC system roles (Company Admin … Temporary Elevated). Mirrors
    /// `db/migrations/core/20260803200001_core_permissions_seed.sql` and
    /// `db/migrations/core/20260803230001_core_enterprise_rbac_seed.sql` as closely as a
    /// single-crate in-memory store can (cross-module codes like `companies.profile.manage`
    /// are omitted since Companies/Users own those catalogs).
    pub fn seeded() -> Self {
        let store = Self::new();
        let now = Utc::now();

        let mut tenant_admin_permissions: Vec<proven_shared::PermissionCode> =
            ALL_CORE_PERMISSIONS.iter().map(|code| (*code).into()).collect();
        tenant_admin_permissions.push(FEATURE_MODULE_ACCESS.into());
        // Place catalog (ADR-0009) — mirrors projects permissions grant to Tenant Admin.
        for code in permissions::PROJECTS_PERMISSIONS {
            tenant_admin_permissions.push((*code).into());
        }

        let admin_role = RoleDefinition {
            id: system_tenant_admin_role_id(),
            tenant_id: None,
            name: "Tenant Admin".to_string(),
            kind: RoleKind::System,
            status: RoleStatus::Active,
            permissions: tenant_admin_permissions,
            created_at: now,
            updated_at: now,
            version: 1,
        };

        let mut safety_coordinator_permissions: Vec<&str> = SAFETY_PERMISSIONS.to_vec();
        safety_coordinator_permissions
            .extend_from_slice(&[PROJECTS_PROJECT_READ, permissions::DOCUMENTS_DOCUMENT_READ]);

        let mut equipment_manager_permissions: Vec<&str> = EQUIPMENT_PERMISSIONS.to_vec();
        equipment_manager_permissions
            .extend_from_slice(&[PROJECTS_PROJECT_READ, FEATURE_MODULE_ACCESS]);

        let mut training_admin_permissions: Vec<&str> = TRAINING_PERMISSIONS.to_vec();
        training_admin_permissions
            .extend_from_slice(&[PROJECTS_PROJECT_READ, FEATURE_MODULE_ACCESS]);

        let mut document_control_permissions: Vec<&str> =
            permissions::DOCUMENTS_PERMISSIONS.to_vec();
        document_control_permissions
            .extend_from_slice(&[APPROVALS_REQUEST_APPROVE, FEATURE_MODULE_ACCESS]);

        let system_roles: Vec<(RoleId, &str, RoleKind, Vec<&str>)> = vec![
            (
                permissions::company_admin_role_id(),
                "Company Admin",
                RoleKind::Company,
                vec![
                    permissions::COMPANY_READ,
                    permissions::COMPANY_MANAGE,
                    FEATURE_MODULE_ACCESS,
                ],
            ),
            (
                permissions::project_admin_role_id(),
                "Project Admin",
                RoleKind::Project,
                vec![
                    PROJECTS_PROJECT_READ,
                    permissions::PROJECTS_PROJECT_MANAGE,
                    permissions::MEMBERSHIP_MANAGE,
                    permissions::SAFETY_ACTIVITY_CREATE,
                    permissions::SAFETY_ACTIVITY_REVIEW,
                    permissions::DOCUMENTS_DOCUMENT_READ,
                    permissions::EQUIPMENT_ASSET_READ,
                    permissions::TRAINING_ASSIGNMENT_MANAGE,
                    FEATURE_MODULE_ACCESS,
                ],
            ),
            (
                permissions::supervisor_role_id(),
                "Supervisor",
                RoleKind::Project,
                vec![
                    PROJECTS_PROJECT_READ,
                    permissions::SAFETY_ACTIVITY_CREATE,
                    permissions::SAFETY_ACTIVITY_REVIEW,
                    permissions::SAFETY_CA_MANAGE,
                    TRAINING_COMPLETION_RECORD,
                    permissions::DOCUMENTS_DOCUMENT_READ,
                    permissions::EQUIPMENT_ASSET_READ,
                    APPROVALS_REQUEST_APPROVE,
                ],
            ),
            (
                permissions::worker_role_id(),
                "Worker",
                RoleKind::Project,
                vec![
                    PROJECTS_PROJECT_READ,
                    permissions::SAFETY_ACTIVITY_CREATE,
                    permissions::SAFETY_ACTIVITY_SUBMIT,
                    permissions::TRAINING_COURSE_READ,
                    permissions::DOCUMENTS_DOCUMENT_READ,
                    permissions::EQUIPMENT_INSPECTION_PERFORM,
                    permissions::APPROVALS_REQUEST_CREATE,
                ],
            ),
            (
                permissions::safety_coordinator_role_id(),
                "Safety Coordinator",
                RoleKind::Project,
                safety_coordinator_permissions,
            ),
            (
                permissions::equipment_manager_role_id(),
                "Equipment Manager",
                RoleKind::Company,
                equipment_manager_permissions,
            ),
            (
                permissions::training_admin_role_id(),
                "Training Admin",
                RoleKind::Company,
                training_admin_permissions,
            ),
            (
                permissions::document_control_role_id(),
                "Document Control",
                RoleKind::Company,
                document_control_permissions,
            ),
            (
                permissions::temporary_elevated_role_id(),
                "Temporary Elevated",
                RoleKind::Temporary,
                vec![
                    permissions::EQUIPMENT_READINESS_OVERRIDE,
                    permissions::DOCUMENTS_VERSION_PUBLISH,
                    permissions::DOCUMENTS_ACL_MANAGE,
                    permissions::APPROVALS_REQUEST_APPROVE,
                    permissions::APPROVALS_REQUEST_REJECT,
                    permissions::APPROVALS_POLICY_MANAGE,
                    permissions::SAFETY_INCIDENT_MANAGE,
                    permissions::SAFETY_CA_MANAGE,
                    permissions::OVERRIDE_MANAGE,
                ],
            ),
        ];

        if let Ok(mut state) = store.state.write() {
            state.roles.insert(admin_role.id.as_uuid(), admin_role);
            for (id, name, kind, perms) in system_roles {
                let role = RoleDefinition {
                    id,
                    tenant_id: None,
                    name: name.to_string(),
                    kind,
                    status: RoleStatus::Active,
                    permissions: perms.into_iter().map(|code| code.into()).collect(),
                    created_at: now,
                    updated_at: now,
                    version: 1,
                };
                state.roles.insert(role.id.as_uuid(), role);
            }
        }
        store
    }

    fn read(&self) -> Result<RwLockReadGuard<'_, MemoryState>, CoreError> {
        self.state
            .read()
            .map_err(|_| CoreError::Internal("memory store lock poisoned".into()))
    }

    fn write(&self) -> Result<RwLockWriteGuard<'_, MemoryState>, CoreError> {
        self.state
            .write()
            .map_err(|_| CoreError::Internal("memory store lock poisoned".into()))
    }
}

#[async_trait]
impl TenantRepository for MemoryStore {
    async fn insert(&self, tenant: &Tenant) -> Result<(), CoreError> {
        self.write()?
            .tenants
            .insert(tenant.id.as_uuid(), tenant.clone());
        Ok(())
    }

    async fn get(&self, id: TenantId) -> Result<Option<Tenant>, CoreError> {
        Ok(self.read()?.tenants.get(&id.as_uuid()).cloned())
    }

    async fn update(&self, tenant: &Tenant) -> Result<(), CoreError> {
        self.write()?
            .tenants
            .insert(tenant.id.as_uuid(), tenant.clone());
        Ok(())
    }
}

#[async_trait]
impl CompanyRepository for MemoryStore {
    async fn insert(&self, company: &Company) -> Result<(), CoreError> {
        self.write()?
            .companies
            .insert(company.id.as_uuid(), company.clone());
        Ok(())
    }

    async fn get(&self, id: CompanyId) -> Result<Option<Company>, CoreError> {
        Ok(self.read()?.companies.get(&id.as_uuid()).cloned())
    }
}

#[async_trait]
impl UserRepository for MemoryStore {
    async fn insert(&self, user: &User) -> Result<(), CoreError> {
        self.write()?.users.insert(user.id.as_uuid(), user.clone());
        Ok(())
    }

    async fn get(&self, tenant_id: TenantId, id: UserId) -> Result<Option<User>, CoreError> {
        Ok(self
            .read()?
            .users
            .get(&id.as_uuid())
            .filter(|u| u.tenant_id == tenant_id)
            .cloned())
    }

    async fn get_by_email(
        &self,
        tenant_id: TenantId,
        email: &str,
    ) -> Result<Option<User>, CoreError> {
        let needle = email.to_ascii_lowercase();
        Ok(self
            .read()?
            .users
            .values()
            .find(|u| u.tenant_id == tenant_id && u.email.to_ascii_lowercase() == needle)
            .cloned())
    }

    async fn update(&self, user: &User) -> Result<(), CoreError> {
        self.write()?.users.insert(user.id.as_uuid(), user.clone());
        Ok(())
    }
}

#[async_trait]
impl RoleRepository for MemoryStore {
    async fn insert(&self, role: &RoleDefinition) -> Result<(), CoreError> {
        self.write()?.roles.insert(role.id.as_uuid(), role.clone());
        Ok(())
    }

    async fn get(&self, id: RoleId) -> Result<Option<RoleDefinition>, CoreError> {
        Ok(self.read()?.roles.get(&id.as_uuid()).cloned())
    }
}

#[async_trait]
impl GrantRepository for MemoryStore {
    async fn insert(&self, grant: &AccessGrant) -> Result<(), CoreError> {
        self.write()?
            .grants
            .insert(grant.id.as_uuid(), grant.clone());
        Ok(())
    }

    async fn get(
        &self,
        tenant_id: TenantId,
        id: GrantId,
    ) -> Result<Option<AccessGrant>, CoreError> {
        Ok(self
            .read()?
            .grants
            .get(&id.as_uuid())
            .filter(|g| g.tenant_id == tenant_id)
            .cloned())
    }

    async fn revoke(
        &self,
        tenant_id: TenantId,
        id: GrantId,
        revoked_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let mut state = self.write()?;
        match state.grants.get_mut(&id.as_uuid()) {
            Some(grant) if grant.tenant_id == tenant_id => {
                grant.revoked_at = Some(revoked_at);
                Ok(())
            }
            _ => Err(CoreError::NotFound("access_grant")),
        }
    }

    async fn list_for_user(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<Vec<AccessGrant>, CoreError> {
        Ok(self
            .read()?
            .grants
            .values()
            .filter(|g| g.tenant_id == tenant_id && g.user_id == user_id)
            .cloned()
            .collect())
    }
}

#[async_trait]
impl OverrideRepository for MemoryStore {
    async fn insert(&self, override_: &PermissionOverride) -> Result<(), CoreError> {
        self.write()?
            .overrides
            .insert(override_.id.as_uuid(), override_.clone());
        Ok(())
    }

    async fn get(
        &self,
        tenant_id: TenantId,
        id: PermissionOverrideId,
    ) -> Result<Option<PermissionOverride>, CoreError> {
        Ok(self
            .read()?
            .overrides
            .get(&id.as_uuid())
            .filter(|o| o.tenant_id == tenant_id)
            .cloned())
    }

    async fn list_for_user(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<Vec<PermissionOverride>, CoreError> {
        Ok(self
            .read()?
            .overrides
            .values()
            .filter(|o| o.tenant_id == tenant_id && o.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn revoke(
        &self,
        tenant_id: TenantId,
        id: PermissionOverrideId,
        revoked_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let mut state = self.write()?;
        match state.overrides.get_mut(&id.as_uuid()) {
            Some(override_) if override_.tenant_id == tenant_id => {
                override_.revoked_at = Some(revoked_at);
                Ok(())
            }
            _ => Err(CoreError::NotFound("permission_override")),
        }
    }
}

#[async_trait]
impl ProjectMembershipRepository for MemoryStore {
    async fn insert(&self, membership: &ProjectMembership) -> Result<(), CoreError> {
        self.write()?
            .memberships
            .insert(membership.id.as_uuid(), membership.clone());
        Ok(())
    }

    async fn find_active(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        user_id: UserId,
    ) -> Result<Option<ProjectMembership>, CoreError> {
        Ok(self
            .read()?
            .memberships
            .values()
            .find(|m| {
                m.tenant_id == tenant_id
                    && m.project_id == project_id
                    && m.user_id == Some(user_id)
                    && !matches!(m.status, MembershipStatus::Removed)
            })
            .cloned())
    }

    async fn list_for_user(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<Vec<ProjectMembership>, CoreError> {
        Ok(self
            .read()?
            .memberships
            .values()
            .filter(|m| m.tenant_id == tenant_id && m.user_id == Some(user_id))
            .cloned()
            .collect())
    }
}

#[async_trait]
impl TeamRepository for MemoryStore {
    async fn insert(&self, team: &Team) -> Result<(), CoreError> {
        self.write()?.teams.insert(team.id.as_uuid(), team.clone());
        Ok(())
    }

    async fn get(&self, tenant_id: TenantId, id: TeamId) -> Result<Option<Team>, CoreError> {
        Ok(self
            .read()?
            .teams
            .get(&id.as_uuid())
            .filter(|t| t.tenant_id == tenant_id)
            .cloned())
    }

    async fn add_member(&self, member: &TeamMember) -> Result<(), CoreError> {
        self.write()?.team_members.push(member.clone());
        Ok(())
    }
}

#[async_trait]
impl FileObjectRepository for MemoryStore {
    async fn insert(&self, file: &FileObject) -> Result<(), CoreError> {
        self.write()?.files.insert(file.id.as_uuid(), file.clone());
        Ok(())
    }

    async fn get(
        &self,
        tenant_id: TenantId,
        id: FileObjectId,
    ) -> Result<Option<FileObject>, CoreError> {
        Ok(self
            .read()?
            .files
            .get(&id.as_uuid())
            .filter(|f| f.tenant_id == tenant_id)
            .cloned())
    }

    async fn update(&self, file: &FileObject) -> Result<(), CoreError> {
        self.write()?.files.insert(file.id.as_uuid(), file.clone());
        Ok(())
    }

    async fn list_versions(
        &self,
        tenant_id: TenantId,
        root_file_id: FileObjectId,
    ) -> Result<Vec<FileObject>, CoreError> {
        let root = FileObjectRepository::get(self, tenant_id, root_file_id)
            .await?
            .ok_or(CoreError::NotFound("file_object"))?;
        let lineage_root = root.parent_file_id.unwrap_or(root.id);
        let mut versions: Vec<_> = self
            .read()?
            .files
            .values()
            .filter(|f| f.tenant_id == tenant_id)
            .filter(|f| f.id == lineage_root || f.parent_file_id == Some(lineage_root))
            .cloned()
            .collect();
        versions.sort_by_key(|f| f.content_version);
        Ok(versions)
    }

    async fn list_expired_temporaries(
        &self,
        tenant_id: TenantId,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<FileObject>, CoreError> {
        Ok(self
            .read()?
            .files
            .values()
            .filter(|f| f.tenant_id == tenant_id)
            .filter(|f| f.is_temporary)
            .filter(|f| f.status != FileObjectStatus::Deleted)
            .filter(|f| f.expires_at.map(|exp| exp <= now).unwrap_or(false))
            .cloned()
            .collect())
    }
}

#[async_trait]
impl crate::application::ports::FileShareLinkRepository for MemoryStore {
    async fn insert(&self, link: &crate::domain::FileShareLink) -> Result<(), CoreError> {
        self.write()?
            .file_share_links
            .insert(link.token.clone(), link.clone());
        Ok(())
    }

    async fn get_by_token(
        &self,
        token: &str,
    ) -> Result<Option<crate::domain::FileShareLink>, CoreError> {
        Ok(self.read()?.file_share_links.get(token).cloned())
    }

    async fn update(&self, link: &crate::domain::FileShareLink) -> Result<(), CoreError> {
        self.write()?
            .file_share_links
            .insert(link.token.clone(), link.clone());
        Ok(())
    }
}

/// Whether an [`AuditEntry`] satisfies every populated filter in `query` — shared by the
/// in-memory `search`/`query` implementations.
fn matches_audit_query(entry: &AuditEntry, tenant_id: TenantId, query: &AuditSearchQuery) -> bool {
    if entry.tenant_id != tenant_id {
        return false;
    }
    if let Some(actor) = query.actor_user_id {
        if entry.actor_user_id != Some(actor) {
            return false;
        }
    }
    if let Some(action) = &query.action {
        if &entry.action != action {
            return false;
        }
    }
    if let Some(module_key) = &query.module_key {
        if entry.module_key.as_ref() != Some(module_key) {
            return false;
        }
    }
    if let Some(category) = &query.category {
        if &entry.category != category {
            return false;
        }
    }
    if let Some(project_id) = query.project_id {
        if entry.project_id != Some(project_id) {
            return false;
        }
    }
    if let Some(company_id) = query.company_id {
        if entry.company_id != Some(company_id) {
            return false;
        }
    }
    if let Some(resource_type) = &query.resource_type {
        if &entry.resource_type != resource_type {
            return false;
        }
    }
    if let Some(resource_id) = query.resource_id {
        if entry.resource_id != Some(resource_id) {
            return false;
        }
    }
    if let Some(workflow_instance_id) = query.workflow_instance_id {
        if entry.workflow_instance_id != Some(workflow_instance_id) {
            return false;
        }
    }
    if let Some(signature_package_id) = query.signature_package_id {
        if entry.signature_package_id != Some(signature_package_id) {
            return false;
        }
    }
    if let Some(outcome) = &query.outcome {
        if &entry.outcome != outcome {
            return false;
        }
    }
    if let Some(from) = query.from {
        if entry.occurred_at < from {
            return false;
        }
    }
    if let Some(to) = query.to {
        if entry.occurred_at > to {
            return false;
        }
    }
    if let Some(q) = &query.q {
        let needle = q.to_ascii_lowercase();
        let action_hit = entry.action.to_ascii_lowercase().contains(&needle);
        let payload_hit = entry
            .payload
            .to_string()
            .to_ascii_lowercase()
            .contains(&needle);
        if !action_hit && !payload_hit {
            return false;
        }
    }
    true
}

#[async_trait]
impl AuditRepository for MemoryStore {
    async fn append(&self, entry: &AuditEntry) -> Result<(), CoreError> {
        self.write()?.audit.push(entry.clone());
        Ok(())
    }

    async fn query(
        &self,
        tenant_id: TenantId,
        page: PageRequest,
    ) -> Result<Page<AuditEntry>, CoreError> {
        self.search(tenant_id, &AuditSearchQuery::default(), page)
            .await
    }

    async fn search(
        &self,
        tenant_id: TenantId,
        query: &AuditSearchQuery,
        page: PageRequest,
    ) -> Result<Page<AuditEntry>, CoreError> {
        let state = self.read()?;
        let mut matching: Vec<AuditEntry> = state
            .audit
            .iter()
            .filter(|e| matches_audit_query(e, tenant_id, query))
            .cloned()
            .collect();
        matching.sort_by(|a, b| b.occurred_at.cmp(&a.occurred_at));

        let total = matching.len() as u64;
        let start = page.offset as usize;
        let end = start
            .saturating_add(page.limit as usize)
            .min(matching.len());
        let items = if start >= matching.len() {
            Vec::new()
        } else {
            matching[start..end].to_vec()
        };

        Ok(Page {
            items,
            total,
            limit: page.limit,
            offset: page.offset,
        })
    }

    async fn last_integrity_hash(&self, tenant_id: TenantId) -> Result<Option<String>, CoreError> {
        Ok(self
            .read()?
            .audit
            .iter()
            .rev()
            .find(|e| e.tenant_id == tenant_id)
            .and_then(|e| e.integrity_hash.clone()))
    }

    async fn insert_export_job(&self, job: &AuditExportJob) -> Result<(), CoreError> {
        self.write()?
            .audit_export_jobs
            .insert(job.id, job.clone());
        Ok(())
    }

    async fn update_export_job(&self, job: &AuditExportJob) -> Result<(), CoreError> {
        self.write()?
            .audit_export_jobs
            .insert(job.id, job.clone());
        Ok(())
    }

    async fn get_export_job(
        &self,
        tenant_id: TenantId,
        job_id: Uuid,
    ) -> Result<Option<AuditExportJob>, CoreError> {
        Ok(self
            .read()?
            .audit_export_jobs
            .get(&job_id)
            .filter(|j| j.tenant_id == tenant_id)
            .cloned())
    }

    async fn get_retention_policy(
        &self,
        tenant_id: TenantId,
    ) -> Result<Option<AuditRetentionPolicy>, CoreError> {
        Ok(self
            .read()?
            .audit_retention_policies
            .get(&tenant_id.as_uuid())
            .cloned())
    }

    async fn upsert_retention_policy(
        &self,
        policy: &AuditRetentionPolicy,
    ) -> Result<(), CoreError> {
        self.write()?
            .audit_retention_policies
            .insert(policy.tenant_id.as_uuid(), policy.clone());
        Ok(())
    }
}

#[async_trait]
impl SettingsRepository for MemoryStore {
    async fn get(
        &self,
        tenant_id: TenantId,
        scope_type: SettingScopeType,
        scope_id: Option<Uuid>,
        key: &SettingKey,
    ) -> Result<Option<SettingEntry>, CoreError> {
        Ok(self
            .read()?
            .settings
            .iter()
            .find(|s| {
                s.tenant_id == tenant_id
                    && s.scope_type == scope_type
                    && s.scope_id == scope_id
                    && &s.key == key
            })
            .cloned())
    }

    async fn upsert(&self, entry: &SettingEntry) -> Result<(), CoreError> {
        let mut state = self.write()?;
        if let Some(existing) = state.settings.iter_mut().find(|s| {
            s.tenant_id == entry.tenant_id
                && s.scope_type == entry.scope_type
                && s.scope_id == entry.scope_id
                && s.key == entry.key
        }) {
            *existing = entry.clone();
        } else {
            state.settings.push(entry.clone());
        }
        Ok(())
    }
}

#[async_trait]
impl FlagsRepository for MemoryStore {
    async fn get_flag(&self, key: &FeatureFlagKey) -> Result<Option<FeatureFlag>, CoreError> {
        Ok(self.read()?.flags.get(&key.0).cloned())
    }

    async fn define_flag(&self, flag: &FeatureFlag) -> Result<(), CoreError> {
        self.write()?.flags.insert(flag.key.0.clone(), flag.clone());
        Ok(())
    }

    async fn get_override(
        &self,
        key: &FeatureFlagKey,
        tenant_id: Option<TenantId>,
        user_id: Option<UserId>,
    ) -> Result<Option<bool>, CoreError> {
        let tenant_uuid = tenant_id.map(|t| t.as_uuid());
        let user_uuid = user_id.map(|u| u.as_uuid());
        Ok(self
            .read()?
            .flag_overrides
            .iter()
            .find(|o| o.key == key.0 && o.tenant_id == tenant_uuid && o.user_id == user_uuid)
            .map(|o| o.enabled))
    }

    async fn set_override(
        &self,
        key: &FeatureFlagKey,
        tenant_id: Option<TenantId>,
        user_id: Option<UserId>,
        enabled: bool,
    ) -> Result<(), CoreError> {
        let tenant_uuid = tenant_id.map(|t| t.as_uuid());
        let user_uuid = user_id.map(|u| u.as_uuid());
        let mut state = self.write()?;
        if let Some(existing) = state
            .flag_overrides
            .iter_mut()
            .find(|o| o.key == key.0 && o.tenant_id == tenant_uuid && o.user_id == user_uuid)
        {
            existing.enabled = enabled;
        } else {
            state.flag_overrides.push(FlagOverride {
                key: key.0.clone(),
                tenant_id: tenant_uuid,
                user_id: user_uuid,
                enabled,
            });
        }
        Ok(())
    }
}

#[async_trait]
impl LicenseRepository for MemoryStore {
    async fn insert(
        &self,
        license: &License,
        entitlements: &[ModuleEntitlement],
    ) -> Result<(), CoreError> {
        let mut state = self.write()?;
        state
            .licenses
            .insert(license.tenant_id.as_uuid(), license.clone());
        state
            .entitlements
            .insert(license.id.as_uuid(), entitlements.to_vec());
        Ok(())
    }

    async fn get_current(&self, tenant_id: TenantId) -> Result<Option<License>, CoreError> {
        Ok(self.read()?.licenses.get(&tenant_id.as_uuid()).cloned())
    }

    async fn get_entitlements(
        &self,
        license_id: LicenseId,
    ) -> Result<Vec<ModuleEntitlement>, CoreError> {
        Ok(self
            .read()?
            .entitlements
            .get(&license_id.as_uuid())
            .cloned()
            .unwrap_or_default())
    }
}
