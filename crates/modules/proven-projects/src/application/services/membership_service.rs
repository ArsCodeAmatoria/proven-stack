//! Membership orchestration — validates Place invariants then calls Core `MembershipApi`.

use std::sync::Arc;

use proven_core::application::services::GrantProjectMembershipCommand;
use proven_core::{AuthzApi, MembershipApi, ProjectMembership};
use proven_shared::{PrincipalId, ProjectId, UserId};

use crate::application::ports::{EventPublisher, ProjectRepository};
use crate::application::services::authz::{authorize, project_scope, ActingContext};
use crate::domain::permissions;
use crate::domain::validation::require_non_empty;
use crate::domain::ProjectsError;
use crate::events::{ActorRef, EventEnvelope, ProjectsEvent, ResourceRef};

pub struct AssignProjectMembershipCommand {
    pub project_id: ProjectId,
    pub user_id: UserId,
    pub membership_role: String,
    pub granted_by: Option<UserId>,
}

pub struct MembershipOrchestrationService {
    projects: Arc<dyn ProjectRepository>,
    outbox: Arc<dyn EventPublisher>,
    authz: Arc<dyn AuthzApi>,
    membership: Option<Arc<dyn MembershipApi>>,
}

impl MembershipOrchestrationService {
    pub fn new(
        projects: Arc<dyn ProjectRepository>,
        outbox: Arc<dyn EventPublisher>,
        authz: Arc<dyn AuthzApi>,
        membership: Option<Arc<dyn MembershipApi>>,
    ) -> Self {
        Self {
            projects,
            outbox,
            authz,
            membership,
        }
    }

    pub async fn assign(
        &self,
        ctx: &ActingContext,
        cmd: AssignProjectMembershipCommand,
    ) -> Result<ProjectMembership, ProjectsError> {
        authorize(
            self.authz.as_ref(),
            ctx,
            permissions::PROJECT_MANAGE,
            project_scope(cmd.project_id),
        )
        .await?;

        require_non_empty("membership_role", &cmd.membership_role)?;

        let project = self
            .projects
            .get(ctx.tenant_id, cmd.project_id)
            .await?
            .ok_or_else(|| ProjectsError::not_found("project"))?;

        if !project.status.accepts_membership() {
            return Err(ProjectsError::conflict(
                "cannot assign membership on a closed or archived project",
            ));
        }

        let membership_api = self.membership.as_ref().ok_or_else(|| {
            ProjectsError::Internal(
                "MembershipApi is not wired; use ProjectsModule::with_core".into(),
            )
        })?;

        let membership = membership_api
            .grant_project_membership(GrantProjectMembershipCommand {
                tenant_id: ctx.tenant_id,
                project_id: cmd.project_id,
                user_id: Some(cmd.user_id),
                person_id: None,
                membership_role: cmd.membership_role.clone(),
                granted_by: cmd.granted_by,
            })
            .await
            .map_err(map_core_membership_error)?;

        self.outbox
            .publish(EventEnvelope::new(
                ctx.tenant_id,
                ActorRef::Principal {
                    principal_id: ctx.principal,
                },
                ResourceRef {
                    resource_type: "project_membership".to_string(),
                    resource_id: membership.id.as_uuid(),
                },
                None,
                None,
                ProjectsEvent::ProjectMembershipAssigned {
                    tenant_id: ctx.tenant_id,
                    project_id: cmd.project_id,
                    user_id: cmd.user_id,
                    membership_role: cmd.membership_role,
                },
            ))
            .await?;

        Ok(membership)
    }

    pub async fn is_member(
        &self,
        tenant_id: proven_shared::TenantId,
        project_id: ProjectId,
        principal: PrincipalId,
    ) -> Result<bool, ProjectsError> {
        let membership_api = self.membership.as_ref().ok_or_else(|| {
            ProjectsError::Internal(
                "MembershipApi is not wired; use ProjectsModule::with_core".into(),
            )
        })?;

        membership_api
            .is_project_member(tenant_id, project_id, principal)
            .await
            .map_err(|err| ProjectsError::Internal(format!("membership check failed: {err}")))
    }

    pub async fn list_principal_projects(
        &self,
        tenant_id: proven_shared::TenantId,
        principal: PrincipalId,
    ) -> Result<Vec<ProjectId>, ProjectsError> {
        let membership_api = self.membership.as_ref().ok_or_else(|| {
            ProjectsError::Internal(
                "MembershipApi is not wired; use ProjectsModule::with_core".into(),
            )
        })?;

        membership_api
            .list_principal_projects(tenant_id, principal)
            .await
            .map_err(|err| ProjectsError::Internal(format!("list projects failed: {err}")))
    }
}

fn map_core_membership_error(err: proven_core::CoreError) -> ProjectsError {
    match err {
        proven_core::CoreError::NotFound(_) => ProjectsError::not_found("membership target"),
        proven_core::CoreError::Validation(msg) => ProjectsError::validation(msg),
        proven_core::CoreError::Conflict(msg) => ProjectsError::conflict(msg),
        proven_core::CoreError::Forbidden(_) | proven_core::CoreError::Unauthorized => {
            ProjectsError::forbidden("membership forbidden")
        }
        other => ProjectsError::Internal(format!("core membership error: {other}")),
    }
}
