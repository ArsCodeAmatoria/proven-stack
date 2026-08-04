//! Identity — user invitation and activation (CORE_DOMAIN.md §10.2).

use std::sync::Arc;

use chrono::Utc;
use proven_shared::{TenantId, UserId};

use crate::application::ports::{AuditRepository, EventPublisher, UserRepository};
use crate::application::services::audit_service::{AppendAuditEntryCommand, AuditService};
use crate::domain::{CoreError, User, UserStatus};
use crate::events::{ActorRef, CoreEvent, EventEnvelope, ResourceRef};

pub struct InviteUserCommand {
    pub tenant_id: TenantId,
    pub email: String,
    pub display_name: String,
    pub invited_by: Option<UserId>,
}

pub struct IdentityService {
    users: Arc<dyn UserRepository>,
    audit: Arc<dyn AuditRepository>,
    outbox: Arc<dyn EventPublisher>,
}

impl IdentityService {
    pub fn new(
        users: Arc<dyn UserRepository>,
        audit: Arc<dyn AuditRepository>,
        outbox: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            users,
            audit,
            outbox,
        }
    }

    pub async fn invite_user(&self, cmd: InviteUserCommand) -> Result<User, CoreError> {
        if cmd.email.trim().is_empty() {
            return Err(CoreError::validation("email must not be empty"));
        }
        if self
            .users
            .get_by_email(cmd.tenant_id, &cmd.email)
            .await?
            .is_some()
        {
            return Err(CoreError::conflict("email already in use for tenant"));
        }

        let now = Utc::now();
        let user = User {
            id: UserId::new(),
            tenant_id: cmd.tenant_id,
            email: cmd.email.clone(),
            display_name: cmd.display_name,
            status: UserStatus::Invited,
            person_id: None,
            created_at: now,
            updated_at: now,
            version: 1,
        };
        self.users.insert(&user).await?;

        AuditService::new(self.audit.clone())
            .append(AppendAuditEntryCommand {
                tenant_id: cmd.tenant_id,
                actor_user_id: cmd.invited_by,
                actor_type: "user".to_string(),
                action: "core.user.invited".to_string(),
                resource_type: "user".to_string(),
                resource_id: Some(user.id.as_uuid()),
                correlation_id: None,
                causation_id: None,
                payload: serde_json::json!({ "email": cmd.email }),
                category: Some("admin".to_string()),
                ..Default::default()
            })
            .await?;

        self.outbox
            .publish(EventEnvelope::new(
                cmd.tenant_id,
                cmd.invited_by
                    .map(|user_id| ActorRef::User { user_id })
                    .unwrap_or(ActorRef::System),
                ResourceRef {
                    resource_type: "user".to_string(),
                    resource_id: user.id.as_uuid(),
                },
                None,
                None,
                CoreEvent::UserInvited {
                    tenant_id: cmd.tenant_id,
                    user_id: user.id,
                    email: cmd.email,
                },
            ))
            .await?;

        Ok(user)
    }

    pub async fn activate_user(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<User, CoreError> {
        let mut user = self
            .users
            .get(tenant_id, user_id)
            .await?
            .ok_or(CoreError::NotFound("user"))?;

        if user.status != UserStatus::Invited {
            return Err(CoreError::conflict("only invited users can be activated"));
        }

        user.status = UserStatus::Active;
        user.updated_at = Utc::now();
        user.version += 1;
        self.users.update(&user).await?;

        AuditService::new(self.audit.clone())
            .append(AppendAuditEntryCommand {
                tenant_id,
                actor_user_id: Some(user_id),
                actor_type: "user".to_string(),
                action: "core.user.activated".to_string(),
                resource_type: "user".to_string(),
                resource_id: Some(user_id.as_uuid()),
                correlation_id: None,
                causation_id: None,
                payload: serde_json::json!({}),
                category: Some("admin".to_string()),
                ..Default::default()
            })
            .await?;

        self.outbox
            .publish(EventEnvelope::new(
                tenant_id,
                ActorRef::User { user_id },
                ResourceRef {
                    resource_type: "user".to_string(),
                    resource_id: user_id.as_uuid(),
                },
                None,
                None,
                CoreEvent::UserActivated { tenant_id, user_id },
            ))
            .await?;

        Ok(user)
    }

    pub async fn get_user(&self, tenant_id: TenantId, user_id: UserId) -> Result<User, CoreError> {
        self.users
            .get(tenant_id, user_id)
            .await?
            .ok_or(CoreError::NotFound("user"))
    }
}
