//! Shared helper used by every mutating service to append a [`ProfileAuditEntry`] and publish
//! both the domain-specific event and a `ProfileAuditAppended` event (ADR-0006 §8: "Audit
//! History = append-only profile change log in this module").

use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use proven_shared::UserId;

use crate::application::ports::{EventPublisher, ProfileAuditRepository};
use crate::application::services::authz::ActingContext;
use crate::domain::{ProfileAuditEntry, ProfileAuditEntryId, UsersError};
use crate::events::{ActorRef, EventEnvelope, ResourceRef, UsersEvent};

/// Bundles the audit repository + outbox so every service can record a change with one call
/// instead of repeating the envelope/entry boilerplate.
pub struct AuditRecorder {
    audit: Arc<dyn ProfileAuditRepository>,
    outbox: Arc<dyn EventPublisher>,
}

impl AuditRecorder {
    pub fn new(audit: Arc<dyn ProfileAuditRepository>, outbox: Arc<dyn EventPublisher>) -> Self {
        Self { audit, outbox }
    }

    /// Appends a `ProfileAuditEntry` for `user_id`, then publishes `domain_event` followed by a
    /// `ProfileAuditAppended` event referencing the new entry.
    #[allow(clippy::too_many_arguments)]
    pub async fn record(
        &self,
        ctx: &ActingContext,
        user_id: UserId,
        action: &str,
        resource_type: &str,
        resource_id: Option<Uuid>,
        summary: impl Into<String>,
        payload: serde_json::Value,
        domain_event: UsersEvent,
    ) -> Result<(), UsersError> {
        let now = Utc::now();
        let entry = ProfileAuditEntry {
            id: ProfileAuditEntryId::new(),
            tenant_id: ctx.tenant_id,
            user_id,
            actor_user_id: Some(ctx.as_user_id()),
            action: action.to_string(),
            resource_type: resource_type.to_string(),
            resource_id,
            summary: summary.into(),
            payload,
            occurred_at: now,
        };
        self.audit.append(&entry).await?;

        self.outbox
            .publish(EventEnvelope::new(
                ctx.tenant_id,
                ActorRef::Principal {
                    principal_id: ctx.principal,
                },
                ResourceRef {
                    resource_type: resource_type.to_string(),
                    resource_id: resource_id.unwrap_or(user_id.as_uuid()),
                },
                None,
                None,
                domain_event,
            ))
            .await?;

        self.outbox
            .publish(EventEnvelope::new(
                ctx.tenant_id,
                ActorRef::Principal {
                    principal_id: ctx.principal,
                },
                ResourceRef {
                    resource_type: "profile_audit_entry".to_string(),
                    resource_id: entry.id.as_uuid(),
                },
                None,
                None,
                UsersEvent::ProfileAuditAppended {
                    tenant_id: ctx.tenant_id,
                    user_id,
                    entry_id: entry.id,
                },
            ))
            .await?;

        Ok(())
    }
}
