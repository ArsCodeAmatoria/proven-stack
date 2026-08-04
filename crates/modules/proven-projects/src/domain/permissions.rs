//! Stable Projects permission codes — aligned with Core's enterprise RBAC seed
//! (`projects.project.{read,create,manage}`). Append-only; AuthZ via `AuthzApi` (ADR-0003).

pub const PROJECT_READ: &str = "projects.project.read";
pub const PROJECT_CREATE: &str = "projects.project.create";
pub const PROJECT_MANAGE: &str = "projects.project.manage";

/// Catalog of Projects-owned permission codes used by this skeleton.
pub const ALL_PROJECTS_PERMISSIONS: &[&str] = &[PROJECT_READ, PROJECT_CREATE, PROJECT_MANAGE];
