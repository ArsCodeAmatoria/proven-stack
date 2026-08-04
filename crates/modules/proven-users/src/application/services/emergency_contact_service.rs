//! `EmergencyContactService` — a user's emergency contacts (ADR-0006 §3).

use std::sync::Arc;

use chrono::Utc;
use proven_core::AuthzApi;
use proven_shared::UserId;

use crate::application::ports::EmergencyContactRepository;
use crate::application::services::audit_recorder::AuditRecorder;
use crate::application::services::authz::{authorize_self_or_permission, ActingContext};
use crate::domain::permissions;
use crate::domain::validation::{validate_email, validate_non_empty};
use crate::domain::{EmergencyContact, EmergencyContactId, UsersError};
use crate::events::UsersEvent;

pub struct AddEmergencyContactCommand {
    pub user_id: UserId,
    pub full_name: String,
    pub relationship: Option<String>,
    pub phone: String,
    pub email: Option<String>,
    pub is_primary: bool,
}

pub struct UpdateEmergencyContactCommand {
    pub user_id: UserId,
    pub contact_id: EmergencyContactId,
    pub full_name: Option<String>,
    pub relationship: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_primary: Option<bool>,
}

pub struct EmergencyContactService {
    contacts: Arc<dyn EmergencyContactRepository>,
    audit: Arc<AuditRecorder>,
    authz: Arc<dyn AuthzApi>,
}

impl EmergencyContactService {
    pub fn new(
        contacts: Arc<dyn EmergencyContactRepository>,
        audit: Arc<AuditRecorder>,
        authz: Arc<dyn AuthzApi>,
    ) -> Self {
        Self {
            contacts,
            audit,
            authz,
        }
    }

    pub async fn add(
        &self,
        ctx: &ActingContext,
        cmd: AddEmergencyContactCommand,
    ) -> Result<EmergencyContact, UsersError> {
        authorize_self_or_permission(
            self.authz.as_ref(),
            ctx,
            permissions::EMERGENCY_CONTACT_MANAGE,
            cmd.user_id,
        )
        .await?;

        validate_non_empty("full_name", &cmd.full_name)?;
        validate_non_empty("phone", &cmd.phone)?;
        if let Some(email) = &cmd.email {
            validate_email(email)?;
        }

        let now = Utc::now();
        let contact = EmergencyContact {
            id: EmergencyContactId::new(),
            tenant_id: ctx.tenant_id,
            user_id: cmd.user_id,
            full_name: cmd.full_name,
            relationship: cmd.relationship,
            phone: cmd.phone,
            email: cmd.email,
            is_primary: cmd.is_primary,
            created_at: now,
            updated_at: now,
        };
        self.contacts.insert(&contact).await?;

        self.audit
            .record(
                ctx,
                cmd.user_id,
                "emergency_contact_added",
                "emergency_contact",
                Some(contact.id.as_uuid()),
                format!("Added emergency contact {}", contact.full_name),
                serde_json::json!({}),
                UsersEvent::EmergencyContactAdded {
                    tenant_id: ctx.tenant_id,
                    user_id: cmd.user_id,
                    contact_id: contact.id,
                },
            )
            .await?;

        Ok(contact)
    }

    pub async fn list(&self, user_id: UserId) -> Result<Vec<EmergencyContact>, UsersError> {
        self.contacts.list(user_id).await
    }

    pub async fn update(
        &self,
        ctx: &ActingContext,
        cmd: UpdateEmergencyContactCommand,
    ) -> Result<EmergencyContact, UsersError> {
        authorize_self_or_permission(
            self.authz.as_ref(),
            ctx,
            permissions::EMERGENCY_CONTACT_MANAGE,
            cmd.user_id,
        )
        .await?;

        if let Some(email) = &cmd.email {
            validate_email(email)?;
        }
        if let Some(phone) = &cmd.phone {
            validate_non_empty("phone", phone)?;
        }

        let mut contact = self
            .contacts
            .get(cmd.user_id, cmd.contact_id)
            .await?
            .ok_or(UsersError::NotFound("emergency_contact"))?;

        if let Some(full_name) = cmd.full_name {
            validate_non_empty("full_name", &full_name)?;
            contact.full_name = full_name;
        }
        if cmd.relationship.is_some() {
            contact.relationship = cmd.relationship;
        }
        if let Some(phone) = cmd.phone {
            contact.phone = phone;
        }
        if cmd.email.is_some() {
            contact.email = cmd.email;
        }
        if let Some(is_primary) = cmd.is_primary {
            contact.is_primary = is_primary;
        }
        contact.updated_at = Utc::now();
        self.contacts.update(&contact).await?;

        self.audit
            .record(
                ctx,
                cmd.user_id,
                "emergency_contact_updated",
                "emergency_contact",
                Some(contact.id.as_uuid()),
                format!("Updated emergency contact {}", contact.full_name),
                serde_json::json!({}),
                UsersEvent::EmergencyContactUpdated {
                    tenant_id: ctx.tenant_id,
                    user_id: cmd.user_id,
                    contact_id: contact.id,
                },
            )
            .await?;

        Ok(contact)
    }

    pub async fn remove(
        &self,
        ctx: &ActingContext,
        user_id: UserId,
        contact_id: EmergencyContactId,
    ) -> Result<(), UsersError> {
        authorize_self_or_permission(
            self.authz.as_ref(),
            ctx,
            permissions::EMERGENCY_CONTACT_MANAGE,
            user_id,
        )
        .await?;

        self.contacts
            .get(user_id, contact_id)
            .await?
            .ok_or(UsersError::NotFound("emergency_contact"))?;
        self.contacts.remove(user_id, contact_id).await?;

        self.audit
            .record(
                ctx,
                user_id,
                "emergency_contact_removed",
                "emergency_contact",
                Some(contact_id.as_uuid()),
                "Removed emergency contact",
                serde_json::json!({}),
                UsersEvent::EmergencyContactRemoved {
                    tenant_id: ctx.tenant_id,
                    user_id,
                    contact_id,
                },
            )
            .await?;

        Ok(())
    }
}
