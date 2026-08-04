//! `ContactService` — company points of contact (ADR-0005 §3).

use std::sync::Arc;

use chrono::Utc;
use proven_core::AuthzApi;
use proven_shared::{CompanyId, UserId};

use crate::application::ports::{ContactRepository, EventPublisher};
use crate::application::services::authz::{authorize, ActingContext};
use crate::domain::validation::{validate_email, validate_non_empty};
use crate::domain::{
    permissions, BusinessUnitId, CompaniesError, Contact, ContactId, ContactKind,
};
use crate::events::{ActorRef, CompaniesEvent, EventEnvelope, ResourceRef};

pub struct AddContactCommand {
    pub company_id: CompanyId,
    pub business_unit_id: Option<BusinessUnitId>,
    pub kind: ContactKind,
    pub full_name: String,
    pub title: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub user_id: Option<UserId>,
    pub is_primary: bool,
}

pub struct UpdateContactCommand {
    pub company_id: CompanyId,
    pub contact_id: ContactId,
    pub business_unit_id: Option<BusinessUnitId>,
    pub kind: Option<ContactKind>,
    pub full_name: Option<String>,
    pub title: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub user_id: Option<UserId>,
    pub is_primary: Option<bool>,
}

pub struct ContactService {
    contacts: Arc<dyn ContactRepository>,
    outbox: Arc<dyn EventPublisher>,
    authz: Arc<dyn AuthzApi>,
}

impl ContactService {
    pub fn new(
        contacts: Arc<dyn ContactRepository>,
        outbox: Arc<dyn EventPublisher>,
        authz: Arc<dyn AuthzApi>,
    ) -> Self {
        Self {
            contacts,
            outbox,
            authz,
        }
    }

    pub async fn add(
        &self,
        ctx: &ActingContext,
        cmd: AddContactCommand,
    ) -> Result<Contact, CompaniesError> {
        authorize(
            self.authz.as_ref(),
            ctx,
            permissions::CONTACT_MANAGE,
            cmd.company_id,
        )
        .await?;
        validate_non_empty("full_name", &cmd.full_name)?;
        if let Some(email) = &cmd.email {
            validate_email(email)?;
        }

        let now = Utc::now();
        let contact = Contact {
            id: ContactId::new(),
            company_id: cmd.company_id,
            tenant_id: ctx.tenant_id,
            business_unit_id: cmd.business_unit_id,
            kind: cmd.kind,
            full_name: cmd.full_name,
            title: cmd.title,
            email: cmd.email,
            phone: cmd.phone,
            user_id: cmd.user_id,
            is_primary: cmd.is_primary,
            created_at: now,
            updated_at: now,
        };
        self.contacts.insert(&contact).await?;

        self.outbox
            .publish(EventEnvelope::new(
                ctx.tenant_id,
                ActorRef::Principal {
                    principal_id: ctx.principal,
                },
                ResourceRef {
                    resource_type: "contact".to_string(),
                    resource_id: contact.id.as_uuid(),
                },
                None,
                None,
                CompaniesEvent::ContactAdded {
                    tenant_id: ctx.tenant_id,
                    company_id: cmd.company_id,
                    contact_id: contact.id,
                },
            ))
            .await?;

        Ok(contact)
    }

    pub async fn list(&self, company_id: CompanyId) -> Result<Vec<Contact>, CompaniesError> {
        self.contacts.list(company_id).await
    }

    pub async fn update(
        &self,
        ctx: &ActingContext,
        cmd: UpdateContactCommand,
    ) -> Result<Contact, CompaniesError> {
        authorize(
            self.authz.as_ref(),
            ctx,
            permissions::CONTACT_MANAGE,
            cmd.company_id,
        )
        .await?;

        if let Some(email) = &cmd.email {
            validate_email(email)?;
        }

        let mut contact = self
            .contacts
            .get(cmd.company_id, cmd.contact_id)
            .await?
            .ok_or(CompaniesError::NotFound("contact"))?;

        if cmd.business_unit_id.is_some() {
            contact.business_unit_id = cmd.business_unit_id;
        }
        if let Some(kind) = cmd.kind {
            contact.kind = kind;
        }
        if let Some(full_name) = cmd.full_name {
            validate_non_empty("full_name", &full_name)?;
            contact.full_name = full_name;
        }
        if cmd.title.is_some() {
            contact.title = cmd.title;
        }
        if cmd.email.is_some() {
            contact.email = cmd.email;
        }
        if cmd.phone.is_some() {
            contact.phone = cmd.phone;
        }
        if cmd.user_id.is_some() {
            contact.user_id = cmd.user_id;
        }
        if let Some(is_primary) = cmd.is_primary {
            contact.is_primary = is_primary;
        }
        contact.updated_at = Utc::now();
        self.contacts.update(&contact).await?;

        self.outbox
            .publish(EventEnvelope::new(
                ctx.tenant_id,
                ActorRef::Principal {
                    principal_id: ctx.principal,
                },
                ResourceRef {
                    resource_type: "contact".to_string(),
                    resource_id: contact.id.as_uuid(),
                },
                None,
                None,
                CompaniesEvent::ContactUpdated {
                    tenant_id: ctx.tenant_id,
                    company_id: cmd.company_id,
                    contact_id: contact.id,
                },
            ))
            .await?;

        Ok(contact)
    }

    pub async fn remove(
        &self,
        ctx: &ActingContext,
        company_id: CompanyId,
        contact_id: ContactId,
    ) -> Result<(), CompaniesError> {
        authorize(
            self.authz.as_ref(),
            ctx,
            permissions::CONTACT_MANAGE,
            company_id,
        )
        .await?;

        self.contacts
            .get(company_id, contact_id)
            .await?
            .ok_or(CompaniesError::NotFound("contact"))?;
        self.contacts.remove(company_id, contact_id).await?;

        self.outbox
            .publish(EventEnvelope::new(
                ctx.tenant_id,
                ActorRef::Principal {
                    principal_id: ctx.principal,
                },
                ResourceRef {
                    resource_type: "contact".to_string(),
                    resource_id: contact_id.as_uuid(),
                },
                None,
                None,
                CompaniesEvent::ContactRemoved {
                    tenant_id: ctx.tenant_id,
                    company_id,
                    contact_id,
                },
            ))
            .await?;

        Ok(())
    }
}
