//! `MembershipPolicyService` — project membership bindings and teams (CORE_DOMAIN.md §10.4).

use std::sync::Arc;

use chrono::Utc;
use proven_shared::{
    PersonId, PrincipalId, ProjectId, ProjectMembershipId, TeamId, TenantId, UserId,
};

use crate::application::ports::{
    AuditRepository, EventPublisher, ProjectMembershipRepository, TeamRepository,
};
use crate::application::services::audit_service::{AppendAuditEntryCommand, AuditService};
use crate::domain::{CoreError, MembershipStatus, ProjectMembership, Team, TeamStatus};
use crate::events::{ActorRef, CoreEvent, EventEnvelope, ResourceRef};

pub struct GrantProjectMembershipCommand {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub user_id: Option<UserId>,
    pub person_id: Option<PersonId>,
    pub membership_role: String,
    pub granted_by: Option<UserId>,
}

pub struct CreateTeamCommand {
    pub tenant_id: TenantId,
    pub name: String,
    pub project_id: Option<ProjectId>,
}

pub struct MembershipService {
    memberships: Arc<dyn ProjectMembershipRepository>,
    teams: Arc<dyn TeamRepository>,
    audit: Arc<dyn AuditRepository>,
    outbox: Arc<dyn EventPublisher>,
}

impl MembershipService {
    pub fn new(
        memberships: Arc<dyn ProjectMembershipRepository>,
        teams: Arc<dyn TeamRepository>,
        audit: Arc<dyn AuditRepository>,
        outbox: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            memberships,
            teams,
            audit,
            outbox,
        }
    }

    pub async fn grant_project_membership(
        &self,
        cmd: GrantProjectMembershipCommand,
    ) -> Result<ProjectMembership, CoreError> {
        if cmd.user_id.is_none() && cmd.person_id.is_none() {
            return Err(CoreError::validation(
                "membership requires a user_id or person_id",
            ));
        }

        if let Some(user_id) = cmd.user_id {
            if self
                .memberships
                .find_active(cmd.tenant_id, cmd.project_id, user_id)
                .await?
                .is_some()
            {
                return Err(CoreError::conflict(
                    "principal already has an active membership on this project",
                ));
            }
        }

        let now = Utc::now();
        let membership = ProjectMembership {
            id: ProjectMembershipId::new(),
            tenant_id: cmd.tenant_id,
            project_id: cmd.project_id,
            user_id: cmd.user_id,
            person_id: cmd.person_id,
            membership_role: cmd.membership_role,
            status: MembershipStatus::Active,
            created_at: now,
            updated_at: now,
            version: 1,
        };
        self.memberships.insert(&membership).await?;

        AuditService::new(self.audit.clone())
            .append(AppendAuditEntryCommand {
                tenant_id: cmd.tenant_id,
                actor_user_id: cmd.granted_by,
                actor_type: "user".to_string(),
                action: "core.membership.granted".to_string(),
                resource_type: "project_membership".to_string(),
                resource_id: Some(membership.id.as_uuid()),
                correlation_id: None,
                causation_id: None,
                payload: serde_json::json!({ "project_id": membership.project_id }),
                category: Some("authz".to_string()),
                project_id: Some(membership.project_id),
                ..Default::default()
            })
            .await?;

        self.outbox
            .publish(EventEnvelope::new(
                cmd.tenant_id,
                cmd.granted_by
                    .map(|user_id| ActorRef::User { user_id })
                    .unwrap_or(ActorRef::System),
                ResourceRef {
                    resource_type: "project_membership".to_string(),
                    resource_id: membership.id.as_uuid(),
                },
                None,
                None,
                CoreEvent::ProjectMembershipGranted {
                    tenant_id: cmd.tenant_id,
                    membership_id: membership.id,
                    project_id: membership.project_id,
                },
            ))
            .await?;

        Ok(membership)
    }

    pub async fn is_project_member(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        principal: PrincipalId,
    ) -> Result<bool, CoreError> {
        let user_id = UserId::from_uuid(principal.as_uuid());
        Ok(self
            .memberships
            .find_active(tenant_id, project_id, user_id)
            .await?
            .map(|m| {
                matches!(
                    m.status,
                    MembershipStatus::Active | MembershipStatus::Invited
                )
            })
            .unwrap_or(false))
    }

    pub async fn list_principal_projects(
        &self,
        tenant_id: TenantId,
        principal: PrincipalId,
    ) -> Result<Vec<ProjectId>, CoreError> {
        let user_id = UserId::from_uuid(principal.as_uuid());
        let memberships = self.memberships.list_for_user(tenant_id, user_id).await?;
        let mut projects = Vec::new();
        for membership in memberships.into_iter().filter(|m| {
            matches!(
                m.status,
                MembershipStatus::Active | MembershipStatus::Invited
            )
        }) {
            if !projects.contains(&membership.project_id) {
                projects.push(membership.project_id);
            }
        }
        Ok(projects)
    }

    pub async fn create_team(&self, cmd: CreateTeamCommand) -> Result<Team, CoreError> {
        if cmd.name.trim().is_empty() {
            return Err(CoreError::validation("team name must not be empty"));
        }

        let now = Utc::now();
        let team = Team {
            id: TeamId::new(),
            tenant_id: cmd.tenant_id,
            name: cmd.name,
            project_id: cmd.project_id,
            status: TeamStatus::Active,
            created_at: now,
            updated_at: now,
            version: 1,
        };
        self.teams.insert(&team).await?;

        AuditService::new(self.audit.clone())
            .append(AppendAuditEntryCommand {
                tenant_id: cmd.tenant_id,
                actor_user_id: None,
                actor_type: "user".to_string(),
                action: "core.team.created".to_string(),
                resource_type: "team".to_string(),
                resource_id: Some(team.id.as_uuid()),
                correlation_id: None,
                causation_id: None,
                payload: serde_json::json!({ "name": team.name }),
                category: Some("authz".to_string()),
                project_id: team.project_id,
                ..Default::default()
            })
            .await?;

        self.outbox
            .publish(EventEnvelope::new(
                cmd.tenant_id,
                ActorRef::System,
                ResourceRef {
                    resource_type: "team".to_string(),
                    resource_id: team.id.as_uuid(),
                },
                None,
                None,
                CoreEvent::TeamCreated {
                    tenant_id: cmd.tenant_id,
                    team_id: team.id,
                },
            ))
            .await?;

        Ok(team)
    }
}
