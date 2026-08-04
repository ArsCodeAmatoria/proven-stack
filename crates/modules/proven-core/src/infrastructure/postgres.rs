//! Thin SQLx adapters over `db/migrations/core/*.sql` (ADR-0004). Uses `sqlx::query` (not the
//! `query!` macro) so this crate compiles without a live database at build time.
//!
//! Tenancy, Identity, Access (roles/grants), Membership, and Audit are implemented here.
//! Files, Settings, Flags, and License remain TODO — the in-memory store is authoritative for
//! those until a follow-up lands real adapters; see ADR-0004 §4.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use proven_shared::{
    CompanyId, GrantId, Page, PageRequest, PermissionOverrideId, ProjectId, ProjectMembershipId,
    RegionCode, RoleId, SessionId, TenantId, UserId,
};
use sqlx::{PgPool, Postgres, QueryBuilder, Row};
use uuid::Uuid;

use crate::application::ports::{
    AuditRepository, CompanyRepository, GrantRepository, OverrideRepository,
    ProjectMembershipRepository, RoleRepository, TenantRepository, UserRepository,
};
use crate::domain::{
    AccessGrant, AccessScope, AuditEntry, AuditExportJob, AuditRetentionPolicy, AuditSearchQuery,
    Company, CompanyStatus, CompanyType, CoreError, GrantKind, GrantScopeType, MembershipStatus,
    OverrideEffect, PermissionOverride, ProjectMembership, RoleDefinition, RoleKind, RoleStatus,
    Tenant, TenantStatus, User, UserStatus,
};

fn tenant_status_str(status: TenantStatus) -> &'static str {
    match status {
        TenantStatus::Active => "active",
        TenantStatus::Suspended => "suspended",
        TenantStatus::Closed => "closed",
    }
}

fn parse_tenant_status(s: &str) -> Result<TenantStatus, CoreError> {
    match s {
        "active" => Ok(TenantStatus::Active),
        "suspended" => Ok(TenantStatus::Suspended),
        "closed" => Ok(TenantStatus::Closed),
        other => Err(CoreError::Internal(format!(
            "unknown tenant status: {other}"
        ))),
    }
}

fn company_status_str(status: CompanyStatus) -> &'static str {
    match status {
        CompanyStatus::Active => "active",
        CompanyStatus::Deactivated => "deactivated",
    }
}

fn parse_company_status(s: &str) -> Result<CompanyStatus, CoreError> {
    match s {
        "active" => Ok(CompanyStatus::Active),
        "deactivated" => Ok(CompanyStatus::Deactivated),
        other => Err(CoreError::Internal(format!(
            "unknown company status: {other}"
        ))),
    }
}

fn company_type_str(kind: CompanyType) -> &'static str {
    match kind {
        CompanyType::Prime => "prime",
        CompanyType::Subcontractor => "subcontractor",
        CompanyType::Crane => "crane",
        CompanyType::Forming => "forming",
        CompanyType::Civil => "civil",
        CompanyType::Industrial => "industrial",
        CompanyType::Other => "other",
    }
}

fn parse_company_type(s: &str) -> Result<CompanyType, CoreError> {
    match s {
        "prime" => Ok(CompanyType::Prime),
        "subcontractor" => Ok(CompanyType::Subcontractor),
        "crane" => Ok(CompanyType::Crane),
        "forming" => Ok(CompanyType::Forming),
        "civil" => Ok(CompanyType::Civil),
        "industrial" => Ok(CompanyType::Industrial),
        "other" => Ok(CompanyType::Other),
        other => Err(CoreError::Internal(format!(
            "unknown company type: {other}"
        ))),
    }
}

fn user_status_str(status: UserStatus) -> &'static str {
    match status {
        UserStatus::Invited => "invited",
        UserStatus::Active => "active",
        UserStatus::Locked => "locked",
        UserStatus::Deactivated => "deactivated",
    }
}

fn parse_user_status(s: &str) -> Result<UserStatus, CoreError> {
    match s {
        "invited" => Ok(UserStatus::Invited),
        "active" => Ok(UserStatus::Active),
        "locked" => Ok(UserStatus::Locked),
        "deactivated" => Ok(UserStatus::Deactivated),
        other => Err(CoreError::Internal(format!("unknown user status: {other}"))),
    }
}

fn role_kind_str(kind: RoleKind) -> &'static str {
    match kind {
        RoleKind::System => "system",
        RoleKind::TenantCustom => "tenant_custom",
        RoleKind::Membership => "membership",
        RoleKind::Company => "company",
        RoleKind::Project => "project",
        RoleKind::Temporary => "temporary",
    }
}

fn parse_role_kind(s: &str) -> Result<RoleKind, CoreError> {
    match s {
        "system" => Ok(RoleKind::System),
        "tenant_custom" => Ok(RoleKind::TenantCustom),
        "membership" => Ok(RoleKind::Membership),
        "company" => Ok(RoleKind::Company),
        "project" => Ok(RoleKind::Project),
        "temporary" => Ok(RoleKind::Temporary),
        other => Err(CoreError::Internal(format!("unknown role kind: {other}"))),
    }
}

fn parse_role_status(s: &str) -> Result<RoleStatus, CoreError> {
    match s {
        "active" => Ok(RoleStatus::Active),
        "retired" => Ok(RoleStatus::Retired),
        other => Err(CoreError::Internal(format!("unknown role status: {other}"))),
    }
}

fn scope_type_str(scope: GrantScopeType) -> &'static str {
    match scope {
        GrantScopeType::Tenant => "tenant",
        GrantScopeType::OrgUnit => "org_unit",
        GrantScopeType::Company => "company",
        GrantScopeType::Project => "project",
        GrantScopeType::Team => "team",
        GrantScopeType::SelfScope => "self",
    }
}

fn parse_scope_type(s: &str) -> Result<GrantScopeType, CoreError> {
    match s {
        "tenant" => Ok(GrantScopeType::Tenant),
        "org_unit" => Ok(GrantScopeType::OrgUnit),
        "company" => Ok(GrantScopeType::Company),
        "project" => Ok(GrantScopeType::Project),
        "team" => Ok(GrantScopeType::Team),
        "self" => Ok(GrantScopeType::SelfScope),
        other => Err(CoreError::Internal(format!("unknown scope type: {other}"))),
    }
}

fn override_effect_str(effect: OverrideEffect) -> &'static str {
    match effect {
        OverrideEffect::Allow => "allow",
        OverrideEffect::Deny => "deny",
    }
}

fn parse_override_effect(s: &str) -> Result<OverrideEffect, CoreError> {
    match s {
        "allow" => Ok(OverrideEffect::Allow),
        "deny" => Ok(OverrideEffect::Deny),
        other => Err(CoreError::Internal(format!(
            "unknown override effect: {other}"
        ))),
    }
}

fn grant_kind_str(kind: GrantKind) -> &'static str {
    match kind {
        GrantKind::Standard => "standard",
        GrantKind::Delegation => "delegation",
        GrantKind::Temporary => "temporary",
        GrantKind::BreakGlass => "break_glass",
    }
}

fn parse_grant_kind(s: &str) -> Result<GrantKind, CoreError> {
    match s {
        "standard" => Ok(GrantKind::Standard),
        "delegation" => Ok(GrantKind::Delegation),
        "temporary" => Ok(GrantKind::Temporary),
        "break_glass" => Ok(GrantKind::BreakGlass),
        other => Err(CoreError::Internal(format!("unknown grant kind: {other}"))),
    }
}

fn membership_status_str(status: MembershipStatus) -> &'static str {
    match status {
        MembershipStatus::Invited => "invited",
        MembershipStatus::Active => "active",
        MembershipStatus::Suspended => "suspended",
        MembershipStatus::Removed => "removed",
    }
}

fn parse_membership_status(s: &str) -> Result<MembershipStatus, CoreError> {
    match s {
        "invited" => Ok(MembershipStatus::Invited),
        "active" => Ok(MembershipStatus::Active),
        "suspended" => Ok(MembershipStatus::Suspended),
        "removed" => Ok(MembershipStatus::Removed),
        other => Err(CoreError::Internal(format!(
            "unknown membership status: {other}"
        ))),
    }
}

pub struct PgTenantRepository {
    pool: PgPool,
}

impl PgTenantRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TenantRepository for PgTenantRepository {
    async fn insert(&self, tenant: &Tenant) -> Result<(), CoreError> {
        sqlx::query(
            "INSERT INTO core.tenants (id, slug, display_name, region_code, status, created_at, updated_at, version)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(tenant.id.as_uuid())
        .bind(&tenant.slug)
        .bind(&tenant.display_name)
        .bind(&tenant.region_code.0)
        .bind(tenant_status_str(tenant.status))
        .bind(tenant.created_at)
        .bind(tenant.updated_at)
        .bind(tenant.version)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get(&self, id: TenantId) -> Result<Option<Tenant>, CoreError> {
        let row = sqlx::query(
            "SELECT id, slug, display_name, region_code, status, created_at, updated_at, version
             FROM core.tenants WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|row| {
            Ok(Tenant {
                id: TenantId(row.try_get("id")?),
                slug: row.try_get("slug")?,
                display_name: row.try_get("display_name")?,
                region_code: RegionCode(row.try_get("region_code")?),
                status: parse_tenant_status(&row.try_get::<String, _>("status")?)?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
                version: row.try_get("version")?,
            })
        })
        .transpose()
    }

    async fn update(&self, tenant: &Tenant) -> Result<(), CoreError> {
        sqlx::query(
            "UPDATE core.tenants SET slug = $2, display_name = $3, region_code = $4, status = $5,
             updated_at = $6, version = $7 WHERE id = $1",
        )
        .bind(tenant.id.as_uuid())
        .bind(&tenant.slug)
        .bind(&tenant.display_name)
        .bind(&tenant.region_code.0)
        .bind(tenant_status_str(tenant.status))
        .bind(tenant.updated_at)
        .bind(tenant.version)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

pub struct PgCompanyRepository {
    pool: PgPool,
}

impl PgCompanyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CompanyRepository for PgCompanyRepository {
    async fn insert(&self, company: &Company) -> Result<(), CoreError> {
        sqlx::query(
            "INSERT INTO core.companies (id, tenant_id, legal_name, display_name, company_type, status, created_at, updated_at, version)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(company.id.as_uuid())
        .bind(company.tenant_id.as_uuid())
        .bind(&company.legal_name)
        .bind(&company.display_name)
        .bind(company_type_str(company.company_type))
        .bind(company_status_str(company.status))
        .bind(company.created_at)
        .bind(company.updated_at)
        .bind(company.version)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get(&self, id: CompanyId) -> Result<Option<Company>, CoreError> {
        let row = sqlx::query(
            "SELECT id, tenant_id, legal_name, display_name, company_type, status, created_at, updated_at, version
             FROM core.companies WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|row| {
            Ok(Company {
                id: CompanyId(row.try_get("id")?),
                tenant_id: TenantId(row.try_get("tenant_id")?),
                legal_name: row.try_get("legal_name")?,
                display_name: row.try_get("display_name")?,
                company_type: parse_company_type(&row.try_get::<String, _>("company_type")?)?,
                status: parse_company_status(&row.try_get::<String, _>("status")?)?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
                version: row.try_get("version")?,
            })
        })
        .transpose()
    }
}

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_user(row: sqlx::postgres::PgRow) -> Result<User, CoreError> {
        Ok(User {
            id: UserId(row.try_get("id")?),
            tenant_id: TenantId(row.try_get("tenant_id")?),
            email: row.try_get("email")?,
            display_name: row.try_get("display_name")?,
            status: parse_user_status(&row.try_get::<String, _>("status")?)?,
            person_id: row
                .try_get::<Option<Uuid>, _>("person_id")?
                .map(proven_shared::PersonId),
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            version: row.try_get("version")?,
        })
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn insert(&self, user: &User) -> Result<(), CoreError> {
        sqlx::query(
            "INSERT INTO core.users (id, tenant_id, email, display_name, status, person_id, created_at, updated_at, version)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(user.id.as_uuid())
        .bind(user.tenant_id.as_uuid())
        .bind(&user.email)
        .bind(&user.display_name)
        .bind(user_status_str(user.status))
        .bind(user.person_id.map(|p| p.as_uuid()))
        .bind(user.created_at)
        .bind(user.updated_at)
        .bind(user.version)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get(&self, tenant_id: TenantId, id: UserId) -> Result<Option<User>, CoreError> {
        let row = sqlx::query(
            "SELECT id, tenant_id, email, display_name, status, person_id, created_at, updated_at, version
             FROM core.users WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.as_uuid())
        .bind(tenant_id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;
        row.map(Self::row_to_user).transpose()
    }

    async fn get_by_email(
        &self,
        tenant_id: TenantId,
        email: &str,
    ) -> Result<Option<User>, CoreError> {
        let row = sqlx::query(
            "SELECT id, tenant_id, email, display_name, status, person_id, created_at, updated_at, version
             FROM core.users WHERE tenant_id = $1 AND lower(email) = lower($2)",
        )
        .bind(tenant_id.as_uuid())
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;
        row.map(Self::row_to_user).transpose()
    }

    async fn update(&self, user: &User) -> Result<(), CoreError> {
        sqlx::query(
            "UPDATE core.users SET email = $2, display_name = $3, status = $4, person_id = $5,
             updated_at = $6, version = $7 WHERE id = $1",
        )
        .bind(user.id.as_uuid())
        .bind(&user.email)
        .bind(&user.display_name)
        .bind(user_status_str(user.status))
        .bind(user.person_id.map(|p| p.as_uuid()))
        .bind(user.updated_at)
        .bind(user.version)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

pub struct PgRoleRepository {
    pool: PgPool,
}

impl PgRoleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RoleRepository for PgRoleRepository {
    async fn insert(&self, role: &RoleDefinition) -> Result<(), CoreError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO core.roles (id, tenant_id, name, kind, status, created_at, updated_at, version)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(role.id.as_uuid())
        .bind(role.tenant_id.map(|t| t.as_uuid()))
        .bind(&role.name)
        .bind(role_kind_str(role.kind))
        .bind(if role.status == RoleStatus::Active {
            "active"
        } else {
            "retired"
        })
        .bind(role.created_at)
        .bind(role.updated_at)
        .bind(role.version)
        .execute(&mut *tx)
        .await?;

        for permission in &role.permissions {
            sqlx::query(
                "INSERT INTO core.role_permissions (role_id, permission_code) VALUES ($1, $2)
                 ON CONFLICT DO NOTHING",
            )
            .bind(role.id.as_uuid())
            .bind(permission.as_str())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn get(&self, id: RoleId) -> Result<Option<RoleDefinition>, CoreError> {
        let row = sqlx::query(
            "SELECT id, tenant_id, name, kind, status, created_at, updated_at, version
             FROM core.roles WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else { return Ok(None) };

        let permission_rows =
            sqlx::query("SELECT permission_code FROM core.role_permissions WHERE role_id = $1")
                .bind(id.as_uuid())
                .fetch_all(&self.pool)
                .await?;
        let permissions = permission_rows
            .into_iter()
            .map(|r| {
                r.try_get::<String, _>("permission_code")
                    .map(proven_shared::PermissionCode::new)
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?;

        Ok(Some(RoleDefinition {
            id: RoleId(row.try_get("id")?),
            tenant_id: row.try_get::<Option<Uuid>, _>("tenant_id")?.map(TenantId),
            name: row.try_get("name")?,
            kind: parse_role_kind(&row.try_get::<String, _>("kind")?)?,
            status: parse_role_status(&row.try_get::<String, _>("status")?)?,
            permissions,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            version: row.try_get("version")?,
        }))
    }
}

pub struct PgGrantRepository {
    pool: PgPool,
}

impl PgGrantRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_grant(row: sqlx::postgres::PgRow) -> Result<AccessGrant, CoreError> {
        Ok(AccessGrant {
            id: GrantId(row.try_get("id")?),
            tenant_id: TenantId(row.try_get("tenant_id")?),
            user_id: UserId(row.try_get("user_id")?),
            role_id: RoleId(row.try_get("role_id")?),
            scope: AccessScope {
                scope_type: parse_scope_type(&row.try_get::<String, _>("scope_type")?)?,
                scope_id: row.try_get("scope_id")?,
            },
            grant_kind: parse_grant_kind(&row.try_get::<String, _>("grant_kind")?)?,
            expires_at: row.try_get("expires_at")?,
            revoked_at: row.try_get("revoked_at")?,
            created_at: row.try_get("created_at")?,
            created_by: row.try_get::<Option<Uuid>, _>("created_by")?.map(UserId),
        })
    }
}

#[async_trait]
impl GrantRepository for PgGrantRepository {
    async fn insert(&self, grant: &AccessGrant) -> Result<(), CoreError> {
        sqlx::query(
            "INSERT INTO core.access_grants
             (id, tenant_id, user_id, role_id, scope_type, scope_id, grant_kind, expires_at, revoked_at, created_at, created_by)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        )
        .bind(grant.id.as_uuid())
        .bind(grant.tenant_id.as_uuid())
        .bind(grant.user_id.as_uuid())
        .bind(grant.role_id.as_uuid())
        .bind(scope_type_str(grant.scope.scope_type))
        .bind(grant.scope.scope_id)
        .bind(grant_kind_str(grant.grant_kind))
        .bind(grant.expires_at)
        .bind(grant.revoked_at)
        .bind(grant.created_at)
        .bind(grant.created_by.map(|u| u.as_uuid()))
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get(
        &self,
        tenant_id: TenantId,
        id: GrantId,
    ) -> Result<Option<AccessGrant>, CoreError> {
        let row = sqlx::query(
            "SELECT id, tenant_id, user_id, role_id, scope_type, scope_id, grant_kind, expires_at, revoked_at, created_at, created_by
             FROM core.access_grants WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.as_uuid())
        .bind(tenant_id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;
        row.map(Self::row_to_grant).transpose()
    }

    async fn revoke(
        &self,
        tenant_id: TenantId,
        id: GrantId,
        revoked_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let result = sqlx::query(
            "UPDATE core.access_grants SET revoked_at = $3 WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.as_uuid())
        .bind(tenant_id.as_uuid())
        .bind(revoked_at)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound("access_grant"));
        }
        Ok(())
    }

    async fn list_for_user(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<Vec<AccessGrant>, CoreError> {
        let rows = sqlx::query(
            "SELECT id, tenant_id, user_id, role_id, scope_type, scope_id, grant_kind, expires_at, revoked_at, created_at, created_by
             FROM core.access_grants WHERE tenant_id = $1 AND user_id = $2",
        )
        .bind(tenant_id.as_uuid())
        .bind(user_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(Self::row_to_grant).collect()
    }
}

pub struct PgProjectMembershipRepository {
    pool: PgPool,
}

impl PgProjectMembershipRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_membership(row: sqlx::postgres::PgRow) -> Result<ProjectMembership, CoreError> {
        Ok(ProjectMembership {
            id: ProjectMembershipId(row.try_get("id")?),
            tenant_id: TenantId(row.try_get("tenant_id")?),
            project_id: ProjectId(row.try_get("project_id")?),
            user_id: row.try_get::<Option<Uuid>, _>("user_id")?.map(UserId),
            person_id: row
                .try_get::<Option<Uuid>, _>("person_id")?
                .map(proven_shared::PersonId),
            membership_role: row.try_get("membership_role")?,
            status: parse_membership_status(&row.try_get::<String, _>("status")?)?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            version: row.try_get("version")?,
        })
    }
}

#[async_trait]
impl ProjectMembershipRepository for PgProjectMembershipRepository {
    async fn insert(&self, membership: &ProjectMembership) -> Result<(), CoreError> {
        sqlx::query(
            "INSERT INTO core.project_memberships
             (id, tenant_id, project_id, user_id, person_id, membership_role, status, created_at, updated_at, version)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(membership.id.as_uuid())
        .bind(membership.tenant_id.as_uuid())
        .bind(membership.project_id.as_uuid())
        .bind(membership.user_id.map(|u| u.as_uuid()))
        .bind(membership.person_id.map(|p| p.as_uuid()))
        .bind(&membership.membership_role)
        .bind(membership_status_str(membership.status))
        .bind(membership.created_at)
        .bind(membership.updated_at)
        .bind(membership.version)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_active(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        user_id: UserId,
    ) -> Result<Option<ProjectMembership>, CoreError> {
        let row = sqlx::query(
            "SELECT id, tenant_id, project_id, user_id, person_id, membership_role, status, created_at, updated_at, version
             FROM core.project_memberships
             WHERE tenant_id = $1 AND project_id = $2 AND user_id = $3 AND status <> 'removed'",
        )
        .bind(tenant_id.as_uuid())
        .bind(project_id.as_uuid())
        .bind(user_id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;
        row.map(Self::row_to_membership).transpose()
    }

    async fn list_for_user(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<Vec<ProjectMembership>, CoreError> {
        let rows = sqlx::query(
            "SELECT id, tenant_id, project_id, user_id, person_id, membership_role, status, created_at, updated_at, version
             FROM core.project_memberships WHERE tenant_id = $1 AND user_id = $2",
        )
        .bind(tenant_id.as_uuid())
        .bind(user_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(Self::row_to_membership).collect()
    }
}

pub struct PgAuditRepository {
    pool: PgPool,
}

impl PgAuditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

const AUDIT_ENTRY_COLUMNS: &str = "id, tenant_id, occurred_at, recorded_at, actor_user_id, actor_type, action, resource_type,
     resource_id, correlation_id, causation_id, payload, payload_digest, module_key, category,
     outcome, project_id, company_id, session_id, ip_address, device_id, user_agent,
     workflow_instance_id, signature_package_id, old_value, new_value, changes, retention_class,
     sensitivity, integrity_prev_hash, integrity_hash";

impl PgAuditRepository {
    fn row_to_entry(row: sqlx::postgres::PgRow) -> Result<AuditEntry, CoreError> {
        Ok(AuditEntry {
            id: proven_shared::AuditEntryId(row.try_get("id")?),
            tenant_id: TenantId(row.try_get("tenant_id")?),
            occurred_at: row.try_get("occurred_at")?,
            recorded_at: row.try_get("recorded_at")?,
            actor_user_id: row.try_get::<Option<Uuid>, _>("actor_user_id")?.map(UserId),
            actor_type: row.try_get("actor_type")?,
            action: row.try_get("action")?,
            resource_type: row.try_get("resource_type")?,
            resource_id: row.try_get("resource_id")?,
            correlation_id: row
                .try_get::<Option<Uuid>, _>("correlation_id")?
                .map(proven_shared::CorrelationId),
            causation_id: row
                .try_get::<Option<Uuid>, _>("causation_id")?
                .map(proven_shared::CausationId),
            payload: row.try_get("payload")?,
            payload_digest: row.try_get("payload_digest")?,
            module_key: row.try_get("module_key")?,
            category: row.try_get("category")?,
            outcome: row.try_get("outcome")?,
            project_id: row.try_get::<Option<Uuid>, _>("project_id")?.map(ProjectId),
            company_id: row.try_get::<Option<Uuid>, _>("company_id")?.map(CompanyId),
            session_id: row.try_get::<Option<Uuid>, _>("session_id")?.map(SessionId),
            ip_address: row.try_get("ip_address")?,
            device_id: row.try_get("device_id")?,
            user_agent: row.try_get("user_agent")?,
            workflow_instance_id: row.try_get("workflow_instance_id")?,
            signature_package_id: row.try_get("signature_package_id")?,
            old_value: row.try_get("old_value")?,
            new_value: row.try_get("new_value")?,
            changes: row.try_get("changes")?,
            retention_class: row.try_get("retention_class")?,
            sensitivity: row.try_get("sensitivity")?,
            integrity_prev_hash: row.try_get("integrity_prev_hash")?,
            integrity_hash: row.try_get("integrity_hash")?,
        })
    }

    /// Appends `AND <condition>` clauses for every populated [`AuditSearchQuery`] field. Shared
    /// by the `count(*)` and row-fetching queries in `search`.
    fn push_filters(builder: &mut QueryBuilder<'_, Postgres>, query: &AuditSearchQuery) {
        if let Some(actor) = query.actor_user_id {
            builder.push(" AND actor_user_id = ").push_bind(actor.as_uuid());
        }
        if let Some(action) = &query.action {
            builder.push(" AND action = ").push_bind(action.clone());
        }
        if let Some(module_key) = &query.module_key {
            builder.push(" AND module_key = ").push_bind(module_key.clone());
        }
        if let Some(category) = &query.category {
            builder.push(" AND category = ").push_bind(category.clone());
        }
        if let Some(project_id) = query.project_id {
            builder.push(" AND project_id = ").push_bind(project_id.as_uuid());
        }
        if let Some(company_id) = query.company_id {
            builder.push(" AND company_id = ").push_bind(company_id.as_uuid());
        }
        if let Some(resource_type) = &query.resource_type {
            builder
                .push(" AND resource_type = ")
                .push_bind(resource_type.clone());
        }
        if let Some(resource_id) = query.resource_id {
            builder.push(" AND resource_id = ").push_bind(resource_id);
        }
        if let Some(workflow_instance_id) = query.workflow_instance_id {
            builder
                .push(" AND workflow_instance_id = ")
                .push_bind(workflow_instance_id);
        }
        if let Some(signature_package_id) = query.signature_package_id {
            builder
                .push(" AND signature_package_id = ")
                .push_bind(signature_package_id);
        }
        if let Some(outcome) = &query.outcome {
            builder.push(" AND outcome = ").push_bind(outcome.clone());
        }
        if let Some(from) = query.from {
            builder.push(" AND occurred_at >= ").push_bind(from);
        }
        if let Some(to) = query.to {
            builder.push(" AND occurred_at <= ").push_bind(to);
        }
        if let Some(q) = &query.q {
            let pattern = format!("%{q}%");
            builder
                .push(" AND (action ILIKE ")
                .push_bind(pattern.clone())
                .push(" OR payload::text ILIKE ")
                .push_bind(pattern)
                .push(")");
        }
    }
}

#[async_trait]
impl AuditRepository for PgAuditRepository {
    async fn append(&self, entry: &AuditEntry) -> Result<(), CoreError> {
        sqlx::query(
            "INSERT INTO core.audit_entries
             (id, tenant_id, occurred_at, recorded_at, actor_user_id, actor_type, action, resource_type,
              resource_id, correlation_id, causation_id, payload, payload_digest, module_key, category,
              outcome, project_id, company_id, session_id, ip_address, device_id, user_agent,
              workflow_instance_id, signature_package_id, old_value, new_value, changes,
              retention_class, sensitivity, integrity_prev_hash, integrity_hash)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18,
                     $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31)",
        )
        .bind(entry.id.as_uuid())
        .bind(entry.tenant_id.as_uuid())
        .bind(entry.occurred_at)
        .bind(entry.recorded_at)
        .bind(entry.actor_user_id.map(|u| u.as_uuid()))
        .bind(&entry.actor_type)
        .bind(&entry.action)
        .bind(&entry.resource_type)
        .bind(entry.resource_id)
        .bind(entry.correlation_id.map(|c| c.as_uuid()))
        .bind(entry.causation_id.map(|c| c.as_uuid()))
        .bind(&entry.payload)
        .bind(&entry.payload_digest)
        .bind(&entry.module_key)
        .bind(&entry.category)
        .bind(&entry.outcome)
        .bind(entry.project_id.map(|p| p.as_uuid()))
        .bind(entry.company_id.map(|c| c.as_uuid()))
        .bind(entry.session_id.map(|s| s.as_uuid()))
        .bind(&entry.ip_address)
        .bind(&entry.device_id)
        .bind(&entry.user_agent)
        .bind(entry.workflow_instance_id)
        .bind(entry.signature_package_id)
        .bind(&entry.old_value)
        .bind(&entry.new_value)
        .bind(&entry.changes)
        .bind(&entry.retention_class)
        .bind(&entry.sensitivity)
        .bind(&entry.integrity_prev_hash)
        .bind(&entry.integrity_hash)
        .execute(&self.pool)
        .await?;
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
        let mut count_builder: QueryBuilder<'_, Postgres> =
            QueryBuilder::new("SELECT count(*) FROM core.audit_entries WHERE tenant_id = ");
        count_builder.push_bind(tenant_id.as_uuid());
        Self::push_filters(&mut count_builder, query);
        let total: i64 = count_builder
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await?;

        let mut builder: QueryBuilder<'_, Postgres> = QueryBuilder::new(format!(
            "SELECT {AUDIT_ENTRY_COLUMNS} FROM core.audit_entries WHERE tenant_id = "
        ));
        builder.push_bind(tenant_id.as_uuid());
        Self::push_filters(&mut builder, query);
        builder.push(" ORDER BY occurred_at DESC LIMIT ");
        builder.push_bind(page.limit as i64);
        builder.push(" OFFSET ");
        builder.push_bind(page.offset as i64);

        let rows = builder.build().fetch_all(&self.pool).await?;
        let items = rows
            .into_iter()
            .map(Self::row_to_entry)
            .collect::<Result<Vec<_>, CoreError>>()?;

        Ok(Page {
            items,
            total: total.max(0) as u64,
            limit: page.limit,
            offset: page.offset,
        })
    }

    async fn last_integrity_hash(&self, tenant_id: TenantId) -> Result<Option<String>, CoreError> {
        let row = sqlx::query(
            "SELECT integrity_hash FROM core.audit_entries WHERE tenant_id = $1
             ORDER BY occurred_at DESC, recorded_at DESC LIMIT 1",
        )
        .bind(tenant_id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(row) => Ok(row.try_get::<Option<String>, _>("integrity_hash")?),
            None => Ok(None),
        }
    }

    async fn insert_export_job(&self, job: &AuditExportJob) -> Result<(), CoreError> {
        sqlx::query(
            "INSERT INTO core.audit_export_jobs
             (id, tenant_id, requested_by, status, filter, entry_count, storage_key, error_message, created_at, completed_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(job.id)
        .bind(job.tenant_id.as_uuid())
        .bind(job.requested_by.map(|u| u.as_uuid()))
        .bind(&job.status)
        .bind(&job.filter)
        .bind(job.entry_count)
        .bind(&job.storage_key)
        .bind(&job.error_message)
        .bind(job.created_at)
        .bind(job.completed_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update_export_job(&self, job: &AuditExportJob) -> Result<(), CoreError> {
        sqlx::query(
            "UPDATE core.audit_export_jobs
             SET status = $3, entry_count = $4, storage_key = $5, error_message = $6, completed_at = $7
             WHERE id = $1 AND tenant_id = $2",
        )
        .bind(job.id)
        .bind(job.tenant_id.as_uuid())
        .bind(&job.status)
        .bind(job.entry_count)
        .bind(&job.storage_key)
        .bind(&job.error_message)
        .bind(job.completed_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_export_job(
        &self,
        tenant_id: TenantId,
        job_id: Uuid,
    ) -> Result<Option<AuditExportJob>, CoreError> {
        let row = sqlx::query(
            "SELECT id, tenant_id, requested_by, status, filter, entry_count, storage_key, error_message, created_at, completed_at
             FROM core.audit_export_jobs WHERE id = $1 AND tenant_id = $2",
        )
        .bind(job_id)
        .bind(tenant_id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|row| {
            Ok(AuditExportJob {
                id: row.try_get("id")?,
                tenant_id: TenantId(row.try_get("tenant_id")?),
                requested_by: row.try_get::<Option<Uuid>, _>("requested_by")?.map(UserId),
                status: row.try_get("status")?,
                filter: row.try_get("filter")?,
                entry_count: row.try_get("entry_count")?,
                storage_key: row.try_get("storage_key")?,
                error_message: row.try_get("error_message")?,
                created_at: row.try_get("created_at")?,
                completed_at: row.try_get("completed_at")?,
            })
        })
        .transpose()
    }

    async fn get_retention_policy(
        &self,
        tenant_id: TenantId,
    ) -> Result<Option<AuditRetentionPolicy>, CoreError> {
        let row = sqlx::query(
            "SELECT tenant_id, standard_days, security_days, compliance_days, restricted_days, export_before_purge, updated_at
             FROM core.audit_retention_policies WHERE tenant_id = $1",
        )
        .bind(tenant_id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|row| {
            Ok(AuditRetentionPolicy {
                tenant_id: TenantId(row.try_get("tenant_id")?),
                standard_days: row.try_get("standard_days")?,
                security_days: row.try_get("security_days")?,
                compliance_days: row.try_get("compliance_days")?,
                restricted_days: row.try_get("restricted_days")?,
                export_before_purge: row.try_get("export_before_purge")?,
                updated_at: row.try_get("updated_at")?,
            })
        })
        .transpose()
    }

    async fn upsert_retention_policy(
        &self,
        policy: &AuditRetentionPolicy,
    ) -> Result<(), CoreError> {
        sqlx::query(
            "INSERT INTO core.audit_retention_policies
             (tenant_id, standard_days, security_days, compliance_days, restricted_days, export_before_purge, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (tenant_id) DO UPDATE SET
               standard_days = EXCLUDED.standard_days,
               security_days = EXCLUDED.security_days,
               compliance_days = EXCLUDED.compliance_days,
               restricted_days = EXCLUDED.restricted_days,
               export_before_purge = EXCLUDED.export_before_purge,
               updated_at = EXCLUDED.updated_at",
        )
        .bind(policy.tenant_id.as_uuid())
        .bind(policy.standard_days)
        .bind(policy.security_days)
        .bind(policy.compliance_days)
        .bind(policy.restricted_days)
        .bind(policy.export_before_purge)
        .bind(policy.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

pub struct PgOverrideRepository {
    pool: PgPool,
}

impl PgOverrideRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_override(row: sqlx::postgres::PgRow) -> Result<PermissionOverride, CoreError> {
        Ok(PermissionOverride {
            id: PermissionOverrideId(row.try_get("id")?),
            tenant_id: TenantId(row.try_get("tenant_id")?),
            user_id: UserId(row.try_get("user_id")?),
            permission_code: proven_shared::PermissionCode::new(
                row.try_get::<String, _>("permission_code")?,
            ),
            effect: parse_override_effect(&row.try_get::<String, _>("effect")?)?,
            scope: AccessScope {
                scope_type: parse_scope_type(&row.try_get::<String, _>("scope_type")?)?,
                scope_id: row.try_get("scope_id")?,
            },
            reason: row.try_get("reason")?,
            expires_at: row.try_get("expires_at")?,
            revoked_at: row.try_get("revoked_at")?,
            created_at: row.try_get("created_at")?,
            created_by: row.try_get::<Option<Uuid>, _>("created_by")?.map(UserId),
        })
    }
}

#[async_trait]
impl OverrideRepository for PgOverrideRepository {
    async fn insert(&self, override_: &PermissionOverride) -> Result<(), CoreError> {
        sqlx::query(
            "INSERT INTO core.permission_overrides
             (id, tenant_id, user_id, permission_code, effect, scope_type, scope_id, reason, expires_at, revoked_at, created_at, created_by)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
        )
        .bind(override_.id.as_uuid())
        .bind(override_.tenant_id.as_uuid())
        .bind(override_.user_id.as_uuid())
        .bind(override_.permission_code.as_str())
        .bind(override_effect_str(override_.effect))
        .bind(scope_type_str(override_.scope.scope_type))
        .bind(override_.scope.scope_id)
        .bind(&override_.reason)
        .bind(override_.expires_at)
        .bind(override_.revoked_at)
        .bind(override_.created_at)
        .bind(override_.created_by.map(|u| u.as_uuid()))
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get(
        &self,
        tenant_id: TenantId,
        id: PermissionOverrideId,
    ) -> Result<Option<PermissionOverride>, CoreError> {
        let row = sqlx::query(
            "SELECT id, tenant_id, user_id, permission_code, effect, scope_type, scope_id, reason, expires_at, revoked_at, created_at, created_by
             FROM core.permission_overrides WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.as_uuid())
        .bind(tenant_id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;
        row.map(Self::row_to_override).transpose()
    }

    async fn list_for_user(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<Vec<PermissionOverride>, CoreError> {
        let rows = sqlx::query(
            "SELECT id, tenant_id, user_id, permission_code, effect, scope_type, scope_id, reason, expires_at, revoked_at, created_at, created_by
             FROM core.permission_overrides WHERE tenant_id = $1 AND user_id = $2",
        )
        .bind(tenant_id.as_uuid())
        .bind(user_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(Self::row_to_override).collect()
    }

    async fn revoke(
        &self,
        tenant_id: TenantId,
        id: PermissionOverrideId,
        revoked_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let result = sqlx::query(
            "UPDATE core.permission_overrides SET revoked_at = $3 WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.as_uuid())
        .bind(tenant_id.as_uuid())
        .bind(revoked_at)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound("permission_override"));
        }
        Ok(())
    }
}
