//! `ProjectService` — create, update, archive (ADR-0009 skeleton).

use std::sync::Arc;

use chrono::{NaiveDate, Utc};
use proven_core::{AuthzApi, TenancyApi};
use proven_shared::{CompanyId, ProjectId};

use crate::application::ports::{EventPublisher, ParticipantRepository, ProjectRepository};
use crate::application::services::authz::{authorize, project_scope, tenant_scope, ActingContext};
use crate::domain::permissions;
use crate::domain::validation::{require_code, require_non_empty};
use crate::domain::{
    ParticipantStatus, ParticipationRole, Project, ProjectLocation, ProjectParticipant,
    ProjectStatus, ProjectsError,
};
use crate::events::{ActorRef, EventEnvelope, ProjectsEvent, ResourceRef};

pub struct CreateProjectCommand {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub location: Option<ProjectLocation>,
    pub prime_contractor_company_id: CompanyId,
    pub client_company_id: Option<CompanyId>,
    pub planned_start: Option<NaiveDate>,
    pub planned_end: Option<NaiveDate>,
}

pub struct UpdateProjectCommand {
    pub project_id: ProjectId,
    pub name: Option<String>,
    pub description: Option<String>,
    pub location: Option<ProjectLocation>,
    pub client_company_id: Option<Option<CompanyId>>,
    pub planned_start: Option<Option<NaiveDate>>,
    pub planned_end: Option<Option<NaiveDate>>,
}

pub struct ProjectService {
    projects: Arc<dyn ProjectRepository>,
    participants: Arc<dyn ParticipantRepository>,
    outbox: Arc<dyn EventPublisher>,
    authz: Arc<dyn AuthzApi>,
    tenancy: Option<Arc<dyn TenancyApi>>,
}

impl ProjectService {
    pub fn new(
        projects: Arc<dyn ProjectRepository>,
        participants: Arc<dyn ParticipantRepository>,
        outbox: Arc<dyn EventPublisher>,
        authz: Arc<dyn AuthzApi>,
        tenancy: Option<Arc<dyn TenancyApi>>,
    ) -> Self {
        Self {
            projects,
            participants,
            outbox,
            authz,
            tenancy,
        }
    }

    pub async fn create(
        &self,
        ctx: &ActingContext,
        cmd: CreateProjectCommand,
    ) -> Result<Project, ProjectsError> {
        authorize(
            self.authz.as_ref(),
            ctx,
            permissions::PROJECT_CREATE,
            tenant_scope(),
        )
        .await?;

        require_code(&cmd.code)?;
        require_non_empty("name", &cmd.name)?;

        if let Some(client) = cmd.client_company_id {
            if client == cmd.prime_contractor_company_id {
                return Err(ProjectsError::validation(
                    "client company must differ from prime contractor",
                ));
            }
        }

        if let Some(tenancy) = &self.tenancy {
            tenancy
                .get_company(cmd.prime_contractor_company_id)
                .await
                .map_err(|_| ProjectsError::not_found("prime contractor company"))?;
            if let Some(client_id) = cmd.client_company_id {
                tenancy
                    .get_company(client_id)
                    .await
                    .map_err(|_| ProjectsError::not_found("client company"))?;
            }
        }

        if self
            .projects
            .get_by_code(ctx.tenant_id, &cmd.code)
            .await?
            .is_some()
        {
            return Err(ProjectsError::conflict(format!(
                "project code '{}' already exists in this tenant",
                cmd.code.trim()
            )));
        }

        let now = Utc::now();
        let project = Project {
            id: ProjectId::new(),
            tenant_id: ctx.tenant_id,
            code: cmd.code.trim().to_string(),
            name: cmd.name.trim().to_string(),
            description: cmd.description,
            status: ProjectStatus::Planning,
            location: cmd.location,
            prime_contractor_company_id: cmd.prime_contractor_company_id,
            client_company_id: cmd.client_company_id,
            planned_start: cmd.planned_start,
            planned_end: cmd.planned_end,
            created_at: now,
            updated_at: now,
            version: 1,
        };

        self.projects.insert(&project).await?;

        let prime = ProjectParticipant {
            id: crate::domain::ParticipantId::new(),
            tenant_id: ctx.tenant_id,
            project_id: project.id,
            company_id: cmd.prime_contractor_company_id,
            role: ParticipationRole::Prime,
            status: ParticipantStatus::Active,
            created_at: now,
            updated_at: now,
            version: 1,
        };
        self.participants.insert(&prime).await?;

        if let Some(client_id) = cmd.client_company_id {
            let client = ProjectParticipant {
                id: crate::domain::ParticipantId::new(),
                tenant_id: ctx.tenant_id,
                project_id: project.id,
                company_id: client_id,
                role: ParticipationRole::Client,
                status: ParticipantStatus::Active,
                created_at: now,
                updated_at: now,
                version: 1,
            };
            self.participants.insert(&client).await?;
        }

        self.outbox
            .publish(EventEnvelope::new(
                ctx.tenant_id,
                ActorRef::Principal {
                    principal_id: ctx.principal,
                },
                ResourceRef {
                    resource_type: "project".to_string(),
                    resource_id: project.id.as_uuid(),
                },
                None,
                None,
                ProjectsEvent::ProjectCreated {
                    tenant_id: ctx.tenant_id,
                    project_id: project.id,
                    code: project.code.clone(),
                    prime_contractor_company_id: project.prime_contractor_company_id,
                },
            ))
            .await?;

        Ok(project)
    }

    pub async fn get(
        &self,
        tenant_id: proven_shared::TenantId,
        project_id: ProjectId,
    ) -> Result<Project, ProjectsError> {
        self.projects
            .get(tenant_id, project_id)
            .await?
            .ok_or_else(|| ProjectsError::not_found("project"))
    }

    pub async fn list(
        &self,
        tenant_id: proven_shared::TenantId,
        include_archived: bool,
    ) -> Result<Vec<Project>, ProjectsError> {
        self.projects.list(tenant_id, include_archived).await
    }

    pub async fn list_participants(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<ProjectParticipant>, ProjectsError> {
        self.participants.list_for_project(project_id).await
    }

    pub async fn update(
        &self,
        ctx: &ActingContext,
        cmd: UpdateProjectCommand,
    ) -> Result<Project, ProjectsError> {
        authorize(
            self.authz.as_ref(),
            ctx,
            permissions::PROJECT_MANAGE,
            project_scope(cmd.project_id),
        )
        .await?;

        let mut project = self
            .projects
            .get(ctx.tenant_id, cmd.project_id)
            .await?
            .ok_or_else(|| ProjectsError::not_found("project"))?;

        if project.status.is_archived() {
            return Err(ProjectsError::conflict("archived projects cannot be updated"));
        }

        if let Some(name) = cmd.name {
            require_non_empty("name", &name)?;
            project.name = name.trim().to_string();
        }
        if let Some(description) = cmd.description {
            project.description = Some(description);
        }
        if let Some(location) = cmd.location {
            project.location = Some(location);
        }
        if let Some(client) = cmd.client_company_id {
            if let Some(client_id) = client {
                if client_id == project.prime_contractor_company_id {
                    return Err(ProjectsError::validation(
                        "client company must differ from prime contractor",
                    ));
                }
                if let Some(tenancy) = &self.tenancy {
                    tenancy
                        .get_company(client_id)
                        .await
                        .map_err(|_| ProjectsError::not_found("client company"))?;
                }
            }
            project.client_company_id = client;
        }
        if let Some(start) = cmd.planned_start {
            project.planned_start = start;
        }
        if let Some(end) = cmd.planned_end {
            project.planned_end = end;
        }

        project.updated_at = Utc::now();
        project.version += 1;
        self.projects.update(&project).await?;

        self.outbox
            .publish(EventEnvelope::new(
                ctx.tenant_id,
                ActorRef::Principal {
                    principal_id: ctx.principal,
                },
                ResourceRef {
                    resource_type: "project".to_string(),
                    resource_id: project.id.as_uuid(),
                },
                None,
                None,
                ProjectsEvent::ProjectUpdated {
                    tenant_id: ctx.tenant_id,
                    project_id: project.id,
                },
            ))
            .await?;

        Ok(project)
    }

    pub async fn archive(
        &self,
        ctx: &ActingContext,
        project_id: ProjectId,
    ) -> Result<Project, ProjectsError> {
        authorize(
            self.authz.as_ref(),
            ctx,
            permissions::PROJECT_MANAGE,
            project_scope(project_id),
        )
        .await?;

        let mut project = self
            .projects
            .get(ctx.tenant_id, project_id)
            .await?
            .ok_or_else(|| ProjectsError::not_found("project"))?;

        if project.status.is_archived() {
            return Err(ProjectsError::conflict("project is already archived"));
        }

        project.status = ProjectStatus::Archived;
        project.updated_at = Utc::now();
        project.version += 1;
        self.projects.update(&project).await?;

        self.outbox
            .publish(EventEnvelope::new(
                ctx.tenant_id,
                ActorRef::Principal {
                    principal_id: ctx.principal,
                },
                ResourceRef {
                    resource_type: "project".to_string(),
                    resource_id: project.id.as_uuid(),
                },
                None,
                None,
                ProjectsEvent::ProjectArchived {
                    tenant_id: ctx.tenant_id,
                    project_id: project.id,
                },
            ))
            .await?;

        Ok(project)
    }
}
