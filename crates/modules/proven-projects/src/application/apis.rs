//! In-process public interface (ADR-0009). Other modules talk to Projects only through
//! [`ProjectsApi`].

use std::sync::Arc;

use async_trait::async_trait;
use proven_core::{AuthzApi, MembershipApi, ProjectMembership, TenancyApi};
use proven_shared::{PrincipalId, ProjectId, TenantId};

use crate::application::ports::{EventPublisher, ParticipantRepository, ProjectRepository};
use crate::application::services::{
    ActingContext, AllowAllAuthz, AssignProjectMembershipCommand, CreateProjectCommand,
    MembershipOrchestrationService, ProjectService, UpdateProjectCommand,
};
use crate::domain::{Project, ProjectParticipant, ProjectsError};
use crate::infrastructure::memory::MemoryStore;
use crate::infrastructure::outbox::InMemoryOutbox;

#[async_trait]
pub trait ProjectsApi: Send + Sync {
    async fn create_project(
        &self,
        ctx: ActingContext,
        cmd: CreateProjectCommand,
    ) -> Result<Project, ProjectsError>;

    async fn get_project(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> Result<Project, ProjectsError>;

    async fn list_projects(
        &self,
        tenant_id: TenantId,
        include_archived: bool,
    ) -> Result<Vec<Project>, ProjectsError>;

    async fn update_project(
        &self,
        ctx: ActingContext,
        cmd: UpdateProjectCommand,
    ) -> Result<Project, ProjectsError>;

    async fn archive_project(
        &self,
        ctx: ActingContext,
        project_id: ProjectId,
    ) -> Result<Project, ProjectsError>;

    async fn list_participants(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<ProjectParticipant>, ProjectsError>;

    async fn assign_membership(
        &self,
        ctx: ActingContext,
        cmd: AssignProjectMembershipCommand,
    ) -> Result<ProjectMembership, ProjectsError>;

    async fn is_member(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        principal: PrincipalId,
    ) -> Result<bool, ProjectsError>;

    async fn list_principal_projects(
        &self,
        tenant_id: TenantId,
        principal: PrincipalId,
    ) -> Result<Vec<ProjectId>, ProjectsError>;
}

pub struct ProjectsPorts {
    pub projects: Arc<dyn ProjectRepository>,
    pub participants: Arc<dyn ParticipantRepository>,
    pub outbox: Arc<dyn EventPublisher>,
}

impl ProjectsPorts {
    pub fn in_memory() -> Self {
        let store = Arc::new(MemoryStore::new());
        let outbox = Arc::new(InMemoryOutbox::new());
        Self {
            projects: store.clone(),
            participants: store,
            outbox,
        }
    }
}

pub struct ProjectsServices {
    projects: ProjectService,
    membership: MembershipOrchestrationService,
}

impl ProjectsServices {
    pub fn new(
        ports: ProjectsPorts,
        authz: Arc<dyn AuthzApi>,
        membership: Option<Arc<dyn MembershipApi>>,
        tenancy: Option<Arc<dyn TenancyApi>>,
    ) -> Self {
        Self {
            projects: ProjectService::new(
                ports.projects.clone(),
                ports.participants,
                ports.outbox.clone(),
                authz.clone(),
                tenancy,
            ),
            membership: MembershipOrchestrationService::new(
                ports.projects,
                ports.outbox,
                authz,
                membership,
            ),
        }
    }

    pub fn in_memory_unchecked() -> Self {
        Self::new(ProjectsPorts::in_memory(), Arc::new(AllowAllAuthz), None, None)
    }

    pub fn with_core<C>(ports: ProjectsPorts, core: Arc<C>) -> Self
    where
        C: AuthzApi + MembershipApi + TenancyApi + Send + Sync + 'static,
    {
        let authz: Arc<dyn AuthzApi> = core.clone();
        let membership: Arc<dyn MembershipApi> = core.clone();
        let tenancy: Arc<dyn TenancyApi> = core;
        Self::new(ports, authz, Some(membership), Some(tenancy))
    }
}

#[async_trait]
impl ProjectsApi for ProjectsServices {
    async fn create_project(
        &self,
        ctx: ActingContext,
        cmd: CreateProjectCommand,
    ) -> Result<Project, ProjectsError> {
        self.projects.create(&ctx, cmd).await
    }

    async fn get_project(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> Result<Project, ProjectsError> {
        self.projects.get(tenant_id, project_id).await
    }

    async fn list_projects(
        &self,
        tenant_id: TenantId,
        include_archived: bool,
    ) -> Result<Vec<Project>, ProjectsError> {
        self.projects.list(tenant_id, include_archived).await
    }

    async fn update_project(
        &self,
        ctx: ActingContext,
        cmd: UpdateProjectCommand,
    ) -> Result<Project, ProjectsError> {
        self.projects.update(&ctx, cmd).await
    }

    async fn archive_project(
        &self,
        ctx: ActingContext,
        project_id: ProjectId,
    ) -> Result<Project, ProjectsError> {
        self.projects.archive(&ctx, project_id).await
    }

    async fn list_participants(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<ProjectParticipant>, ProjectsError> {
        self.projects.list_participants(project_id).await
    }

    async fn assign_membership(
        &self,
        ctx: ActingContext,
        cmd: AssignProjectMembershipCommand,
    ) -> Result<ProjectMembership, ProjectsError> {
        self.membership.assign(&ctx, cmd).await
    }

    async fn is_member(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        principal: PrincipalId,
    ) -> Result<bool, ProjectsError> {
        self.membership
            .is_member(tenant_id, project_id, principal)
            .await
    }

    async fn list_principal_projects(
        &self,
        tenant_id: TenantId,
        principal: PrincipalId,
    ) -> Result<Vec<ProjectId>, ProjectsError> {
        self.membership
            .list_principal_projects(tenant_id, principal)
            .await
    }
}
