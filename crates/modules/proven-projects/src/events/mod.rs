//! Integration events published by Projects (ADR-0009). Subjects: `proven.projects.v1.<EventName>`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use proven_shared::{CausationId, CompanyId, CorrelationId, PrincipalId, ProjectId, TenantId, UserId};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum ProjectsEvent {
    ProjectCreated {
        tenant_id: TenantId,
        project_id: ProjectId,
        code: String,
        prime_contractor_company_id: CompanyId,
    },
    ProjectUpdated {
        tenant_id: TenantId,
        project_id: ProjectId,
    },
    ProjectArchived {
        tenant_id: TenantId,
        project_id: ProjectId,
    },
    ProjectMembershipAssigned {
        tenant_id: TenantId,
        project_id: ProjectId,
        user_id: UserId,
        membership_role: String,
    },
}

impl ProjectsEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::ProjectCreated { .. } => "ProjectCreated",
            Self::ProjectUpdated { .. } => "ProjectUpdated",
            Self::ProjectArchived { .. } => "ProjectArchived",
            Self::ProjectMembershipAssigned { .. } => "ProjectMembershipAssigned",
        }
    }

    pub fn subject(&self) -> String {
        format!("proven.projects.v1.{}", self.event_type())
    }
}

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
    pub payload: ProjectsEvent,
}

impl EventEnvelope {
    pub fn new(
        tenant_id: TenantId,
        actor: ActorRef,
        resource: ResourceRef,
        correlation_id: Option<CorrelationId>,
        causation_id: Option<CausationId>,
        payload: ProjectsEvent,
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
