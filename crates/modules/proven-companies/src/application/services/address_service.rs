//! `AddressService` — company addresses (ADR-0005 §3).

use std::sync::Arc;

use chrono::Utc;
use proven_core::AuthzApi;
use proven_shared::CompanyId;

use crate::application::ports::{AddressRepository, EventPublisher};
use crate::application::services::authz::{authorize, ActingContext};
use crate::domain::validation::{validate_country_code, validate_non_empty};
use crate::domain::{permissions, Address, AddressId, AddressKind, BusinessUnitId, CompaniesError};
use crate::events::{ActorRef, CompaniesEvent, EventEnvelope, ResourceRef};

pub struct AddAddressCommand {
    pub company_id: CompanyId,
    pub business_unit_id: Option<BusinessUnitId>,
    pub kind: AddressKind,
    pub line1: String,
    pub line2: Option<String>,
    pub city: String,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub country_code: String,
    pub is_primary: bool,
}

pub struct UpdateAddressCommand {
    pub company_id: CompanyId,
    pub address_id: AddressId,
    pub business_unit_id: Option<BusinessUnitId>,
    pub kind: Option<AddressKind>,
    pub line1: Option<String>,
    pub line2: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub country_code: Option<String>,
    pub is_primary: Option<bool>,
}

pub struct AddressService {
    addresses: Arc<dyn AddressRepository>,
    outbox: Arc<dyn EventPublisher>,
    authz: Arc<dyn AuthzApi>,
}

impl AddressService {
    pub fn new(
        addresses: Arc<dyn AddressRepository>,
        outbox: Arc<dyn EventPublisher>,
        authz: Arc<dyn AuthzApi>,
    ) -> Self {
        Self {
            addresses,
            outbox,
            authz,
        }
    }

    pub async fn add(
        &self,
        ctx: &ActingContext,
        cmd: AddAddressCommand,
    ) -> Result<Address, CompaniesError> {
        authorize(
            self.authz.as_ref(),
            ctx,
            permissions::ADDRESS_MANAGE,
            cmd.company_id,
        )
        .await?;
        validate_non_empty("line1", &cmd.line1)?;
        validate_non_empty("city", &cmd.city)?;
        validate_country_code(&cmd.country_code)?;

        let now = Utc::now();
        let address = Address {
            id: AddressId::new(),
            company_id: cmd.company_id,
            tenant_id: ctx.tenant_id,
            business_unit_id: cmd.business_unit_id,
            kind: cmd.kind,
            line1: cmd.line1,
            line2: cmd.line2,
            city: cmd.city,
            region: cmd.region,
            postal_code: cmd.postal_code,
            country_code: cmd.country_code,
            is_primary: cmd.is_primary,
            created_at: now,
            updated_at: now,
        };
        self.addresses.insert(&address).await?;

        self.outbox
            .publish(EventEnvelope::new(
                ctx.tenant_id,
                ActorRef::Principal {
                    principal_id: ctx.principal,
                },
                ResourceRef {
                    resource_type: "address".to_string(),
                    resource_id: address.id.as_uuid(),
                },
                None,
                None,
                CompaniesEvent::AddressAdded {
                    tenant_id: ctx.tenant_id,
                    company_id: cmd.company_id,
                    address_id: address.id,
                },
            ))
            .await?;

        Ok(address)
    }

    pub async fn list(&self, company_id: CompanyId) -> Result<Vec<Address>, CompaniesError> {
        self.addresses.list(company_id).await
    }

    pub async fn update(
        &self,
        ctx: &ActingContext,
        cmd: UpdateAddressCommand,
    ) -> Result<Address, CompaniesError> {
        authorize(
            self.authz.as_ref(),
            ctx,
            permissions::ADDRESS_MANAGE,
            cmd.company_id,
        )
        .await?;

        let mut address = self
            .addresses
            .get(cmd.company_id, cmd.address_id)
            .await?
            .ok_or(CompaniesError::NotFound("address"))?;

        if cmd.business_unit_id.is_some() {
            address.business_unit_id = cmd.business_unit_id;
        }
        if let Some(kind) = cmd.kind {
            address.kind = kind;
        }
        if let Some(line1) = cmd.line1 {
            validate_non_empty("line1", &line1)?;
            address.line1 = line1;
        }
        if cmd.line2.is_some() {
            address.line2 = cmd.line2;
        }
        if let Some(city) = cmd.city {
            validate_non_empty("city", &city)?;
            address.city = city;
        }
        if cmd.region.is_some() {
            address.region = cmd.region;
        }
        if cmd.postal_code.is_some() {
            address.postal_code = cmd.postal_code;
        }
        if let Some(country_code) = cmd.country_code {
            validate_country_code(&country_code)?;
            address.country_code = country_code;
        }
        if let Some(is_primary) = cmd.is_primary {
            address.is_primary = is_primary;
        }
        address.updated_at = Utc::now();
        self.addresses.update(&address).await?;

        self.outbox
            .publish(EventEnvelope::new(
                ctx.tenant_id,
                ActorRef::Principal {
                    principal_id: ctx.principal,
                },
                ResourceRef {
                    resource_type: "address".to_string(),
                    resource_id: address.id.as_uuid(),
                },
                None,
                None,
                CompaniesEvent::AddressUpdated {
                    tenant_id: ctx.tenant_id,
                    company_id: cmd.company_id,
                    address_id: address.id,
                },
            ))
            .await?;

        Ok(address)
    }

    pub async fn remove(
        &self,
        ctx: &ActingContext,
        company_id: CompanyId,
        address_id: AddressId,
    ) -> Result<(), CompaniesError> {
        authorize(
            self.authz.as_ref(),
            ctx,
            permissions::ADDRESS_MANAGE,
            company_id,
        )
        .await?;

        self.addresses
            .get(company_id, address_id)
            .await?
            .ok_or(CompaniesError::NotFound("address"))?;
        self.addresses.remove(company_id, address_id).await?;

        self.outbox
            .publish(EventEnvelope::new(
                ctx.tenant_id,
                ActorRef::Principal {
                    principal_id: ctx.principal,
                },
                ResourceRef {
                    resource_type: "address".to_string(),
                    resource_id: address_id.as_uuid(),
                },
                None,
                None,
                CompaniesEvent::AddressRemoved {
                    tenant_id: ctx.tenant_id,
                    company_id,
                    address_id,
                },
            ))
            .await?;

        Ok(())
    }
}
