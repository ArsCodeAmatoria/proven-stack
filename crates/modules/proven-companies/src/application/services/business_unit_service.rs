//! `BusinessUnitService` — company-scoped business unit hierarchy (ADR-0005 §3).

use std::sync::Arc;

use chrono::Utc;
use proven_core::AuthzApi;
use proven_shared::CompanyId;
use uuid::Uuid;

use crate::application::ports::{BusinessUnitRepository, EventPublisher};
use crate::application::services::authz::{authorize, ActingContext};
use crate::domain::{permissions, BusinessUnit, BusinessUnitId, BusinessUnitStatus, CompaniesError};
use crate::events::{ActorRef, CompaniesEvent, EventEnvelope, ResourceRef};

pub struct CreateBusinessUnitCommand {
    pub company_id: CompanyId,
    pub name: String,
    pub code: Option<String>,
    pub parent_id: Option<BusinessUnitId>,
    pub org_unit_id: Option<Uuid>,
}

pub struct UpdateBusinessUnitCommand {
    pub company_id: CompanyId,
    pub business_unit_id: BusinessUnitId,
    pub name: Option<String>,
    pub code: Option<String>,
    pub parent_id: Option<BusinessUnitId>,
    pub org_unit_id: Option<Uuid>,
}

pub struct BusinessUnitService {
    units: Arc<dyn BusinessUnitRepository>,
    outbox: Arc<dyn EventPublisher>,
    authz: Arc<dyn AuthzApi>,
}

impl BusinessUnitService {
    pub fn new(
        units: Arc<dyn BusinessUnitRepository>,
        outbox: Arc<dyn EventPublisher>,
        authz: Arc<dyn AuthzApi>,
    ) -> Self {
        Self {
            units,
            outbox,
            authz,
        }
    }

    async fn assert_same_company_parent(
        &self,
        company_id: CompanyId,
        parent_id: Option<BusinessUnitId>,
    ) -> Result<(), CompaniesError> {
        if let Some(parent_id) = parent_id {
            self.units
                .get(company_id, parent_id)
                .await?
                .ok_or_else(|| {
                    CompaniesError::validation(
                        "parent business unit must belong to the same company",
                    )
                })?;
        }
        Ok(())
    }

    pub async fn create(
        &self,
        ctx: &ActingContext,
        cmd: CreateBusinessUnitCommand,
    ) -> Result<BusinessUnit, CompaniesError> {
        authorize(
            self.authz.as_ref(),
            ctx,
            permissions::UNIT_MANAGE,
            cmd.company_id,
        )
        .await?;
        crate::domain::validation::validate_non_empty("name", &cmd.name)?;
        self.assert_same_company_parent(cmd.company_id, cmd.parent_id)
            .await?;

        let now = Utc::now();
        let unit = BusinessUnit {
            id: BusinessUnitId::new(),
            company_id: cmd.company_id,
            tenant_id: ctx.tenant_id,
            parent_id: cmd.parent_id,
            org_unit_id: cmd.org_unit_id,
            name: cmd.name,
            code: cmd.code,
            status: BusinessUnitStatus::Active,
            created_at: now,
            updated_at: now,
            version: 1,
        };
        self.units.insert(&unit).await?;

        self.outbox
            .publish(EventEnvelope::new(
                ctx.tenant_id,
                ActorRef::Principal {
                    principal_id: ctx.principal,
                },
                ResourceRef {
                    resource_type: "business_unit".to_string(),
                    resource_id: unit.id.as_uuid(),
                },
                None,
                None,
                CompaniesEvent::BusinessUnitCreated {
                    tenant_id: ctx.tenant_id,
                    company_id: cmd.company_id,
                    business_unit_id: unit.id,
                },
            ))
            .await?;

        Ok(unit)
    }

    pub async fn list(&self, company_id: CompanyId) -> Result<Vec<BusinessUnit>, CompaniesError> {
        self.units.list(company_id).await
    }

    pub async fn update(
        &self,
        ctx: &ActingContext,
        cmd: UpdateBusinessUnitCommand,
    ) -> Result<BusinessUnit, CompaniesError> {
        authorize(
            self.authz.as_ref(),
            ctx,
            permissions::UNIT_MANAGE,
            cmd.company_id,
        )
        .await?;

        if let Some(parent_id) = cmd.parent_id {
            if parent_id == cmd.business_unit_id {
                return Err(CompaniesError::validation(
                    "a business unit cannot be its own parent",
                ));
            }
        }
        self.assert_same_company_parent(cmd.company_id, cmd.parent_id)
            .await?;

        let mut unit = self
            .units
            .get(cmd.company_id, cmd.business_unit_id)
            .await?
            .ok_or(CompaniesError::NotFound("business_unit"))?;

        if let Some(name) = cmd.name {
            crate::domain::validation::validate_non_empty("name", &name)?;
            unit.name = name;
        }
        if let Some(code) = cmd.code {
            unit.code = Some(code);
        }
        if cmd.parent_id.is_some() {
            unit.parent_id = cmd.parent_id;
        }
        if cmd.org_unit_id.is_some() {
            unit.org_unit_id = cmd.org_unit_id;
        }
        unit.updated_at = Utc::now();
        unit.version += 1;
        self.units.update(&unit).await?;

        self.outbox
            .publish(EventEnvelope::new(
                ctx.tenant_id,
                ActorRef::Principal {
                    principal_id: ctx.principal,
                },
                ResourceRef {
                    resource_type: "business_unit".to_string(),
                    resource_id: unit.id.as_uuid(),
                },
                None,
                None,
                CompaniesEvent::BusinessUnitUpdated {
                    tenant_id: ctx.tenant_id,
                    company_id: cmd.company_id,
                    business_unit_id: unit.id,
                },
            ))
            .await?;

        Ok(unit)
    }

    pub async fn archive(
        &self,
        ctx: &ActingContext,
        company_id: CompanyId,
        business_unit_id: BusinessUnitId,
    ) -> Result<BusinessUnit, CompaniesError> {
        authorize(self.authz.as_ref(), ctx, permissions::UNIT_MANAGE, company_id).await?;

        let mut unit = self
            .units
            .get(company_id, business_unit_id)
            .await?
            .ok_or(CompaniesError::NotFound("business_unit"))?;
        unit.status = BusinessUnitStatus::Archived;
        unit.updated_at = Utc::now();
        unit.version += 1;
        self.units.update(&unit).await?;

        self.outbox
            .publish(EventEnvelope::new(
                ctx.tenant_id,
                ActorRef::Principal {
                    principal_id: ctx.principal,
                },
                ResourceRef {
                    resource_type: "business_unit".to_string(),
                    resource_id: unit.id.as_uuid(),
                },
                None,
                None,
                CompaniesEvent::BusinessUnitArchived {
                    tenant_id: ctx.tenant_id,
                    company_id,
                    business_unit_id,
                },
            ))
            .await?;

        Ok(unit)
    }
}
