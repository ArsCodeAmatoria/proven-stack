//! Repository / outbound ports for Projects (ADR-0009).

use async_trait::async_trait;

use proven_shared::{ProjectId, TenantId};

use crate::domain::{Project, ProjectParticipant, ProjectsError};
use crate::events::EventEnvelope;

#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn insert(&self, project: &Project) -> Result<(), ProjectsError>;
    async fn get(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> Result<Option<Project>, ProjectsError>;
    async fn get_by_code(
        &self,
        tenant_id: TenantId,
        code: &str,
    ) -> Result<Option<Project>, ProjectsError>;
    async fn update(&self, project: &Project) -> Result<(), ProjectsError>;
    async fn list(
        &self,
        tenant_id: TenantId,
        include_archived: bool,
    ) -> Result<Vec<Project>, ProjectsError>;
}

#[async_trait]
pub trait ParticipantRepository: Send + Sync {
    async fn insert(&self, participant: &ProjectParticipant) -> Result<(), ProjectsError>;
    async fn list_for_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<ProjectParticipant>, ProjectsError>;
}

#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, envelope: EventEnvelope) -> Result<(), ProjectsError>;
}
