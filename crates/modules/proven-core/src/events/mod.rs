//! Integration events (CORE_DOMAIN.md §9) — published on `proven.core.v1.<event>`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use proven_shared::{
    AuditEntryId, CausationId, CompanyId, CorrelationId, FeatureFlagKey, FileObjectId, GrantId,
    LicenseId, PermissionCode, PermissionOverrideId, ProjectId, ProjectMembershipId, RoleId,
    SettingKey, TeamId, TenantId, UserId,
};

/// Who performed the action that produced this event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "actor_type", rename_all = "snake_case")]
pub enum ActorRef {
    User { user_id: UserId },
    System,
}

/// What the event is about.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRef {
    pub resource_type: String,
    pub resource_id: Uuid,
}

/// Domain events published by Core (CORE_DOMAIN.md §9.1-§9.6 — representative subset).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum CoreEvent {
    TenantProvisioned {
        tenant_id: TenantId,
        owner_company_id: CompanyId,
        admin_user_id: UserId,
    },
    UserInvited {
        tenant_id: TenantId,
        user_id: UserId,
        email: String,
    },
    UserActivated {
        tenant_id: TenantId,
        user_id: UserId,
    },
    AccessGranted {
        tenant_id: TenantId,
        grant_id: GrantId,
        user_id: UserId,
        role_id: RoleId,
    },
    AccessRevoked {
        tenant_id: TenantId,
        grant_id: GrantId,
    },
    PermissionOverrideCreated {
        tenant_id: TenantId,
        override_id: PermissionOverrideId,
        user_id: UserId,
        permission_code: PermissionCode,
    },
    PermissionOverrideRevoked {
        tenant_id: TenantId,
        override_id: PermissionOverrideId,
    },
    ProjectMembershipGranted {
        tenant_id: TenantId,
        membership_id: ProjectMembershipId,
        project_id: ProjectId,
    },
    ProjectMembershipRevoked {
        tenant_id: TenantId,
        membership_id: ProjectMembershipId,
    },
    TeamCreated {
        tenant_id: TenantId,
        team_id: TeamId,
    },
    FileUploadIntentCreated {
        tenant_id: TenantId,
        file_id: FileObjectId,
    },
    FileObjectAvailable {
        tenant_id: TenantId,
        file_id: FileObjectId,
    },
    FileObjectQuarantined {
        tenant_id: TenantId,
        file_id: FileObjectId,
    },
    FileObjectScanPending {
        tenant_id: TenantId,
        file_id: FileObjectId,
    },
    FileObjectDeleted {
        tenant_id: TenantId,
        file_id: FileObjectId,
    },
    AuditEntryAppended {
        tenant_id: TenantId,
        audit_entry_id: AuditEntryId,
    },
    AuditExportRequested {
        job_id: Uuid,
        tenant_id: TenantId,
    },
    AuditExportCompleted {
        job_id: Uuid,
        tenant_id: TenantId,
        entry_count: i32,
        storage_key: String,
    },
    SettingsChanged {
        tenant_id: TenantId,
        key: SettingKey,
    },
    FeatureFlagChanged {
        key: FeatureFlagKey,
    },
    LicenseActivated {
        tenant_id: TenantId,
        license_id: LicenseId,
    },
}

impl CoreEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::TenantProvisioned { .. } => "TenantProvisioned",
            Self::UserInvited { .. } => "UserInvited",
            Self::UserActivated { .. } => "UserActivated",
            Self::AccessGranted { .. } => "AccessGranted",
            Self::AccessRevoked { .. } => "AccessRevoked",
            Self::PermissionOverrideCreated { .. } => "PermissionOverrideCreated",
            Self::PermissionOverrideRevoked { .. } => "PermissionOverrideRevoked",
            Self::ProjectMembershipGranted { .. } => "ProjectMembershipGranted",
            Self::ProjectMembershipRevoked { .. } => "ProjectMembershipRevoked",
            Self::TeamCreated { .. } => "TeamCreated",
            Self::FileUploadIntentCreated { .. } => "FileUploadIntentCreated",
            Self::FileObjectAvailable { .. } => "FileObjectAvailable",
            Self::FileObjectQuarantined { .. } => "FileObjectQuarantined",
            Self::FileObjectScanPending { .. } => "FileObjectScanPending",
            Self::FileObjectDeleted { .. } => "FileObjectDeleted",
            Self::AuditEntryAppended { .. } => "AuditEntryAppended",
            Self::AuditExportRequested { .. } => "AuditExportRequested",
            Self::AuditExportCompleted { .. } => "AuditExportCompleted",
            Self::SettingsChanged { .. } => "SettingsChanged",
            Self::FeatureFlagChanged { .. } => "FeatureFlagChanged",
            Self::LicenseActivated { .. } => "LicenseActivated",
        }
    }
}

/// Standard Core event envelope (CORE_DOMAIN.md §9.7).
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
    pub payload: CoreEvent,
}

impl EventEnvelope {
    pub fn new(
        tenant_id: TenantId,
        actor: ActorRef,
        resource: ResourceRef,
        correlation_id: Option<CorrelationId>,
        causation_id: Option<CausationId>,
        payload: CoreEvent,
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
