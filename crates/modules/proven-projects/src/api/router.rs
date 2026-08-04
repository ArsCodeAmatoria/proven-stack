//! HTTP routes for Projects.

use axum::routing::{get, post};
use axum::Router;

use crate::api::handlers;
use crate::ProjectsModule;

pub fn router(module: ProjectsModule) -> Router {
    Router::new()
        .route(
            "/api/v1/projects",
            get(handlers::list_projects).post(handlers::create_project),
        )
        .route("/api/v1/projects/mine", get(handlers::list_my_projects))
        .route(
            "/api/v1/projects/{project_id}",
            get(handlers::get_project).patch(handlers::update_project),
        )
        .route(
            "/api/v1/projects/{project_id}/archive",
            post(handlers::archive_project),
        )
        .route(
            "/api/v1/projects/{project_id}/participants",
            get(handlers::list_participants),
        )
        .route(
            "/api/v1/projects/{project_id}/memberships",
            post(handlers::assign_membership),
        )
        .with_state(module)
}
