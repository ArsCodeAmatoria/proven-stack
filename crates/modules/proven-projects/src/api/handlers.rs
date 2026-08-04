//! Axum handlers — thin adapters over `ProjectsServices`.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use proven_core::ProjectMembership;
use proven_shared::{AppError, CompanyId, ProblemDetails, ProjectId, UserId};
use uuid::Uuid;

use crate::api::dto::{
    AssignMembershipRequest, CreateProjectRequest, ListProjectsQuery, UpdateProjectRequest,
};
use crate::api::extractors::ProjectsPrincipal;
use crate::application::services::{
    AssignProjectMembershipCommand, CreateProjectCommand, UpdateProjectCommand,
};
use crate::application::ProjectsApi;
use crate::domain::{Project, ProjectParticipant, ProjectsError};
use crate::ProjectsModule;

pub struct ApiError(ProjectsError);

impl From<ProjectsError> for ApiError {
    fn from(value: ProjectsError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let app_error: AppError = self.0.into();
        let status = StatusCode::from_u16(app_error.status_code())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        if matches!(app_error, AppError::Internal(_)) {
            tracing::error!(error = %app_error, "projects internal API error");
        }
        (status, Json(ProblemDetails::from(&app_error))).into_response()
    }
}

type ApiResult<T> = Result<Json<T>, ApiError>;

pub async fn create_project(
    State(module): State<ProjectsModule>,
    principal: ProjectsPrincipal,
    Json(body): Json<CreateProjectRequest>,
) -> ApiResult<Project> {
    let project = module
        .services
        .create_project(
            principal.acting_context(),
            CreateProjectCommand {
                code: body.code,
                name: body.name,
                description: body.description,
                location: body.location,
                prime_contractor_company_id: CompanyId::from_uuid(body.prime_contractor_company_id),
                client_company_id: body.client_company_id.map(CompanyId::from_uuid),
                planned_start: body.planned_start,
                planned_end: body.planned_end,
            },
        )
        .await?;
    Ok(Json(project))
}

pub async fn list_projects(
    State(module): State<ProjectsModule>,
    principal: ProjectsPrincipal,
    Query(query): Query<ListProjectsQuery>,
) -> ApiResult<Vec<Project>> {
    let projects = module
        .services
        .list_projects(principal.tenant_id, query.include_archived)
        .await?;
    Ok(Json(projects))
}

pub async fn get_project(
    State(module): State<ProjectsModule>,
    principal: ProjectsPrincipal,
    Path(project_id): Path<Uuid>,
) -> ApiResult<Project> {
    let project = module
        .services
        .get_project(
            principal.tenant_id,
            ProjectId::from_uuid(project_id),
        )
        .await?;
    Ok(Json(project))
}

pub async fn update_project(
    State(module): State<ProjectsModule>,
    principal: ProjectsPrincipal,
    Path(project_id): Path<Uuid>,
    Json(body): Json<UpdateProjectRequest>,
) -> ApiResult<Project> {
    let project = module
        .services
        .update_project(
            principal.acting_context(),
            UpdateProjectCommand {
                project_id: ProjectId::from_uuid(project_id),
                name: body.name,
                description: body.description,
                location: body.location,
                client_company_id: body
                    .client_company_id
                    .map(|opt| opt.map(CompanyId::from_uuid)),
                planned_start: body.planned_start,
                planned_end: body.planned_end,
            },
        )
        .await?;
    Ok(Json(project))
}

pub async fn archive_project(
    State(module): State<ProjectsModule>,
    principal: ProjectsPrincipal,
    Path(project_id): Path<Uuid>,
) -> ApiResult<Project> {
    let project = module
        .services
        .archive_project(principal.acting_context(), ProjectId::from_uuid(project_id))
        .await?;
    Ok(Json(project))
}

pub async fn list_participants(
    State(module): State<ProjectsModule>,
    Path(project_id): Path<Uuid>,
) -> ApiResult<Vec<ProjectParticipant>> {
    let participants = module
        .services
        .list_participants(ProjectId::from_uuid(project_id))
        .await?;
    Ok(Json(participants))
}

pub async fn assign_membership(
    State(module): State<ProjectsModule>,
    principal: ProjectsPrincipal,
    Path(project_id): Path<Uuid>,
    Json(body): Json<AssignMembershipRequest>,
) -> ApiResult<ProjectMembership> {
    let membership = module
        .services
        .assign_membership(
            principal.acting_context(),
            AssignProjectMembershipCommand {
                project_id: ProjectId::from_uuid(project_id),
                user_id: UserId::from_uuid(body.user_id),
                membership_role: body.membership_role,
                granted_by: Some(principal.user_id),
            },
        )
        .await?;
    Ok(Json(membership))
}

pub async fn list_my_projects(
    State(module): State<ProjectsModule>,
    principal: ProjectsPrincipal,
) -> ApiResult<Vec<ProjectId>> {
    let ids = module
        .services
        .list_principal_projects(principal.tenant_id, principal.principal_id())
        .await?;
    Ok(Json(ids))
}
