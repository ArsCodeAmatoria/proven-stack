//! Integration events published by Users (ADR-0006). Each variant's subject follows
//! `proven.users.v1.<EventName>` (e.g. `proven.users.v1.UserProfileEnsured`), mirroring the
//! `proven.core.v1.*` / `proven.companies.v1.*` conventions used by other modules.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use proven_shared::{CausationId, CorrelationId, FileObjectId, PrincipalId, TenantId, UserId};

use crate::domain::{EmergencyContactId, ProfileAuditEntryId, UserKind, UserKindAssignmentId};

/// Who performed the action that produced this event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "actor_type", rename_all = "snake_case")]
pub enum ActorRef {
    Principal { principal_id: PrincipalId },
    System,
}

/// What the event is about.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRef {
    pub resource_type: String,
    pub resource_id: Uuid,
}

/// Domain events published by Users (ADR-0006).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum UsersEvent {
    UserProfileEnsured {
        tenant_id: TenantId,
        user_id: UserId,
    },
    UserProfileUpdated {
        tenant_id: TenantId,
        user_id: UserId,
    },
    UserProfileArchived {
        tenant_id: TenantId,
        user_id: UserId,
    },
    UserKindAssigned {
        tenant_id: TenantId,
        user_id: UserId,
        assignment_id: UserKindAssignmentId,
        kind: UserKind,
    },
    UserKindRemoved {
        tenant_id: TenantId,
        user_id: UserId,
        kind: UserKind,
    },
    AvatarUpdated {
        tenant_id: TenantId,
        user_id: UserId,
        file_object_id: Option<FileObjectId>,
    },
    LocaleUpdated {
        tenant_id: TenantId,
        user_id: UserId,
    },
    AccessibilityUpdated {
        tenant_id: TenantId,
        user_id: UserId,
    },
    NotificationPreferencesUpdated {
        tenant_id: TenantId,
        user_id: UserId,
    },
    AuthenticationProfileUpdated {
        tenant_id: TenantId,
        user_id: UserId,
    },
    DigitalSignatureProfileUpdated {
        tenant_id: TenantId,
        user_id: UserId,
    },
    EmergencyContactAdded {
        tenant_id: TenantId,
        user_id: UserId,
        contact_id: EmergencyContactId,
    },
    EmergencyContactUpdated {
        tenant_id: TenantId,
        user_id: UserId,
        contact_id: EmergencyContactId,
    },
    EmergencyContactRemoved {
        tenant_id: TenantId,
        user_id: UserId,
        contact_id: EmergencyContactId,
    },
    UserSettingUpserted {
        tenant_id: TenantId,
        user_id: UserId,
        key: String,
    },
    ProfileAuditAppended {
        tenant_id: TenantId,
        user_id: UserId,
        entry_id: ProfileAuditEntryId,
    },
}

impl UsersEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::UserProfileEnsured { .. } => "UserProfileEnsured",
            Self::UserProfileUpdated { .. } => "UserProfileUpdated",
            Self::UserProfileArchived { .. } => "UserProfileArchived",
            Self::UserKindAssigned { .. } => "UserKindAssigned",
            Self::UserKindRemoved { .. } => "UserKindRemoved",
            Self::AvatarUpdated { .. } => "AvatarUpdated",
            Self::LocaleUpdated { .. } => "LocaleUpdated",
            Self::AccessibilityUpdated { .. } => "AccessibilityUpdated",
            Self::NotificationPreferencesUpdated { .. } => "NotificationPreferencesUpdated",
            Self::AuthenticationProfileUpdated { .. } => "AuthenticationProfileUpdated",
            Self::DigitalSignatureProfileUpdated { .. } => "DigitalSignatureProfileUpdated",
            Self::EmergencyContactAdded { .. } => "EmergencyContactAdded",
            Self::EmergencyContactUpdated { .. } => "EmergencyContactUpdated",
            Self::EmergencyContactRemoved { .. } => "EmergencyContactRemoved",
            Self::UserSettingUpserted { .. } => "UserSettingUpserted",
            Self::ProfileAuditAppended { .. } => "ProfileAuditAppended",
        }
    }

    /// NATS-style subject this event is published on, e.g.
    /// `proven.users.v1.UserProfileEnsured`.
    pub fn subject(&self) -> String {
        format!("proven.users.v1.{}", self.event_type())
    }
}

/// Standard Users event envelope, structurally aligned with `proven_core::events::EventEnvelope`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: Uuid,
    pub event_type: String,
    pub event_version: u32,
    pub occurred_at: DateTime<Utc>,
    pub tenant_id: TenantId,
    pub actor: ActorRef,
    pub correlation_id: Option<CorrelationId>,
    pub causation_id: Option<CausationId>,
    pub resource: ResourceRef,
    pub payload: UsersEvent,
}

impl EventEnvelope {
    pub fn new(
        tenant_id: TenantId,
        actor: ActorRef,
        resource: ResourceRef,
        correlation_id: Option<CorrelationId>,
        causation_id: Option<CausationId>,
        payload: UsersEvent,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type: payload.event_type().to_string(),
            event_version: 1,
            occurred_at: Utc::now(),
            tenant_id,
            actor,
            correlation_id,
            causation_id,
            resource,
            payload,
        }
    }
}
