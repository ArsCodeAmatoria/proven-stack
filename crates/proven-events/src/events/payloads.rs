//! Initial event payloads + helpers to build envelopes.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use proven_shared::{CompanyId, FileObjectId, ProjectId, TenantId, UserId};

use crate::envelope::{ActorRef, EventEnvelope, ResourceRef};
use crate::naming::EventSubject;

/// Discriminated set of the first integration events.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event_name", content = "data")]
pub enum InitialEvent {
    CompanyCreated(CompanyCreated),
    UserCreated(UserCreated),
    ProjectCreated(ProjectCreated),
    AuditRecorded(AuditRecorded),
    FileUploaded(FileUploaded),
}

impl InitialEvent {
    pub fn event_name(&self) -> &'static str {
        match self {
            Self::CompanyCreated(_) => "CompanyCreated",
            Self::UserCreated(_) => "UserCreated",
            Self::ProjectCreated(_) => "ProjectCreated",
            Self::AuditRecorded(_) => "AuditRecorded",
            Self::FileUploaded(_) => "FileUploaded",
        }
    }

    pub fn module(&self) -> &'static str {
        match self {
            Self::CompanyCreated(_) => "core",
            Self::UserCreated(_) => "core",
            Self::ProjectCreated(_) => "projects",
            Self::AuditRecorded(_) => "core",
            Self::FileUploaded(_) => "core",
        }
    }

    pub fn subject_def(&self) -> EventSubject {
        EventSubject::new(self.module(), 1, self.event_name())
    }

    pub fn into_envelope(
        self,
        tenant_id: TenantId,
        actor: ActorRef,
    ) -> Result<EventEnvelope, crate::error::EventError> {
        let (resource, project_id, payload) = match &self {
            Self::CompanyCreated(p) => (
                ResourceRef {
                    resource_type: "company".into(),
                    resource_id: p.company_id.as_uuid(),
                },
                None,
                serde_json::to_value(p)?,
            ),
            Self::UserCreated(p) => (
                ResourceRef {
                    resource_type: "user".into(),
                    resource_id: p.user_id.as_uuid(),
                },
                None,
                serde_json::to_value(p)?,
            ),
            Self::ProjectCreated(p) => (
                ResourceRef {
                    resource_type: "project".into(),
                    resource_id: p.project_id.as_uuid(),
                },
                Some(p.project_id.as_uuid()),
                serde_json::to_value(p)?,
            ),
            Self::AuditRecorded(p) => (
                ResourceRef {
                    resource_type: "audit_entry".into(),
                    resource_id: p.audit_entry_id,
                },
                p.project_id.map(|id| id.as_uuid()),
                serde_json::to_value(p)?,
            ),
            Self::FileUploaded(p) => (
                ResourceRef {
                    resource_type: "file_object".into(),
                    resource_id: p.file_id.as_uuid(),
                },
                None,
                serde_json::to_value(p)?,
            ),
        };

        let mut envelope = EventEnvelope::new(
            self.module(),
            self.event_name(),
            tenant_id,
            actor,
            resource,
            payload,
        );
        if let Some(project_id) = project_id {
            envelope = envelope.with_project(project_id);
        }
        Ok(envelope)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyCreated {
    pub company_id: CompanyId,
    pub legal_name: String,
    pub company_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserCreated {
    pub user_id: UserId,
    pub email: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectCreated {
    pub project_id: ProjectId,
    pub code: String,
    pub name: String,
    pub prime_contractor_company_id: CompanyId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditRecorded {
    pub audit_entry_id: Uuid,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub project_id: Option<ProjectId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileUploaded {
    pub file_id: FileObjectId,
    pub object_class: String,
    pub content_type: Option<String>,
    pub byte_size: Option<i64>,
    pub storage_key: String,
}

/// Well-known subjects for the initial catalog.
pub mod subjects {
    use crate::naming::EventSubject;

    pub const COMPANY_CREATED: EventSubject = EventSubject::new("core", 1, "CompanyCreated");
    pub const USER_CREATED: EventSubject = EventSubject::new("core", 1, "UserCreated");
    pub const PROJECT_CREATED: EventSubject = EventSubject::new("projects", 1, "ProjectCreated");
    pub const AUDIT_RECORDED: EventSubject = EventSubject::new("core", 1, "AuditRecorded");
    pub const FILE_UPLOADED: EventSubject = EventSubject::new("core", 1, "FileUploaded");
}
