//! `TenantProvisioningService` — bootstrap tenant + owner company + admin user + license
//! (CORE_DOMAIN.md §8, §10.1).

use std::sync::Arc;

use chrono::{Duration, Utc};
use proven_shared::{CompanyId, ModuleKey, RegionCode, TenantId, UserId};

use crate::application::ports::{
    AuditRepository, CompanyRepository, EventPublisher, GrantRepository, LicenseRepository,
    RoleRepository, TenantRepository, UserRepository,
};
use crate::application::services::audit_service::{AppendAuditEntryCommand, AuditService};
use crate::domain::permissions::system_tenant_admin_role_id;
use crate::domain::{
    AccessGrant, AccessScope, Company, CompanyStatus, CompanyType, CoreError, GrantKind, License,
    LicenseStatus, ModuleEntitlement, Tenant, TenantStatus, User, UserStatus,
};
use crate::events::{ActorRef, CoreEvent, EventEnvelope, ResourceRef};

pub struct ProvisionTenantCommand {
    pub slug: String,
    pub display_name: String,
    pub region_code: RegionCode,
    pub owner_company_name: String,
    pub owner_company_type: CompanyType,
    pub admin_email: String,
    pub admin_display_name: String,
    pub seats_limit: i32,
}

pub struct ProvisionTenantResult {
    pub tenant: Tenant,
    pub owner_company: Company,
    pub admin_user: User,
    pub license: License,
}

pub struct RegisterCompanyCommand {
    pub tenant_id: TenantId,
    pub legal_name: String,
    pub display_name: String,
    pub company_type: CompanyType,
}

pub struct TenancyService {
    tenants: Arc<dyn TenantRepository>,
    companies: Arc<dyn CompanyRepository>,
    users: Arc<dyn UserRepository>,
    roles: Arc<dyn RoleRepository>,
    grants: Arc<dyn GrantRepository>,
    licenses: Arc<dyn LicenseRepository>,
    audit: Arc<dyn AuditRepository>,
    outbox: Arc<dyn EventPublisher>,
}

impl TenancyService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenants: Arc<dyn TenantRepository>,
        companies: Arc<dyn CompanyRepository>,
        users: Arc<dyn UserRepository>,
        roles: Arc<dyn RoleRepository>,
        grants: Arc<dyn GrantRepository>,
        licenses: Arc<dyn LicenseRepository>,
        audit: Arc<dyn AuditRepository>,
        outbox: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            tenants,
            companies,
            users,
            roles,
            grants,
            licenses,
            audit,
            outbox,
        }
    }

    pub async fn provision_tenant(
        &self,
        cmd: ProvisionTenantCommand,
    ) -> Result<ProvisionTenantResult, CoreError> {
        if cmd.slug.trim().is_empty() {
            return Err(CoreError::validation("slug must not be empty"));
        }

        let now = Utc::now();

        let tenant = Tenant {
            id: TenantId::new(),
            slug: cmd.slug,
            display_name: cmd.display_name,
            region_code: cmd.region_code,
            status: TenantStatus::Active,
            created_at: now,
            updated_at: now,
            version: 1,
        };
        self.tenants.insert(&tenant).await?;

        let owner_company = Company {
            id: CompanyId::new(),
            tenant_id: tenant.id,
            legal_name: cmd.owner_company_name.clone(),
            display_name: cmd.owner_company_name,
            company_type: cmd.owner_company_type,
            status: CompanyStatus::Active,
            created_at: now,
            updated_at: now,
            version: 1,
        };
        self.companies.insert(&owner_company).await?;

        let admin_user = User {
            id: UserId::new(),
            tenant_id: tenant.id,
            email: cmd.admin_email,
            display_name: cmd.admin_display_name,
            status: UserStatus::Active,
            person_id: None,
            created_at: now,
            updated_at: now,
            version: 1,
        };
        self.users.insert(&admin_user).await?;

        let tenant_admin_role_id = system_tenant_admin_role_id();
        let role = self.roles.get(tenant_admin_role_id).await?.ok_or_else(|| {
            CoreError::Internal("system Tenant Admin role missing — store was not seeded".into())
        })?;

        let admin_grant = AccessGrant {
            id: proven_shared::GrantId::new(),
            tenant_id: tenant.id,
            user_id: admin_user.id,
            role_id: role.id,
            scope: AccessScope::tenant(),
            grant_kind: GrantKind::Standard,
            expires_at: None,
            revoked_at: None,
            created_at: now,
            created_by: None,
        };
        self.grants.insert(&admin_grant).await?;

        let license = License {
            id: proven_shared::LicenseId::new(),
            tenant_id: tenant.id,
            status: LicenseStatus::Trial,
            plan_code: "trial".to_string(),
            seats_limit: cmd.seats_limit,
            starts_at: now,
            ends_at: Some(now + Duration::days(30)),
            created_at: now,
            updated_at: now,
            version: 1,
        };
        let entitlements = vec![ModuleEntitlement {
            license_id: license.id,
            module_key: ModuleKey("core".to_string()),
            enabled: true,
        }];
        self.licenses.insert(&license, &entitlements).await?;

        let audit_entry = AuditService::new(self.audit.clone())
            .append(AppendAuditEntryCommand {
                tenant_id: tenant.id,
                actor_user_id: Some(admin_user.id),
                actor_type: "system".to_string(),
                action: "core.tenant.provisioned".to_string(),
                resource_type: "tenant".to_string(),
                resource_id: Some(tenant.id.as_uuid()),
                correlation_id: None,
                causation_id: None,
                payload: serde_json::json!({
                    "tenant_id": tenant.id,
                    "owner_company_id": owner_company.id,
                    "admin_user_id": admin_user.id,
                }),
                category: Some("admin".to_string()),
                ..Default::default()
            })
            .await?;

        self.outbox
            .publish(EventEnvelope::new(
                tenant.id,
                ActorRef::User {
                    user_id: admin_user.id,
                },
                ResourceRef {
                    resource_type: "tenant".to_string(),
                    resource_id: tenant.id.as_uuid(),
                },
                None,
                Some(proven_shared::CausationId::from_uuid(
                    audit_entry.id.as_uuid(),
                )),
                CoreEvent::TenantProvisioned {
                    tenant_id: tenant.id,
                    owner_company_id: owner_company.id,
                    admin_user_id: admin_user.id,
                },
            ))
            .await?;

        Ok(ProvisionTenantResult {
            tenant,
            owner_company,
            admin_user,
            license,
        })
    }

    pub async fn get_tenant(&self, id: TenantId) -> Result<Tenant, CoreError> {
        self.tenants
            .get(id)
            .await?
            .ok_or(CoreError::NotFound("tenant"))
    }

    pub async fn register_company(
        &self,
        cmd: RegisterCompanyCommand,
    ) -> Result<Company, CoreError> {
        let tenant = self.get_tenant(cmd.tenant_id).await?;
        if tenant.status != TenantStatus::Active {
            return Err(CoreError::Forbidden("tenant is not active".into()));
        }

        let now = Utc::now();
        let company = Company {
            id: CompanyId::new(),
            tenant_id: cmd.tenant_id,
            legal_name: cmd.legal_name,
            display_name: cmd.display_name,
            company_type: cmd.company_type,
            status: CompanyStatus::Active,
            created_at: now,
            updated_at: now,
            version: 1,
        };
        self.companies.insert(&company).await?;
        Ok(company)
    }

    pub async fn get_company(&self, id: CompanyId) -> Result<Company, CoreError> {
        self.companies
            .get(id)
            .await?
            .ok_or(CoreError::NotFound("company"))
    }
}
