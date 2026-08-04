//! Proven Projects — Place (construction undertaking) module skeleton (ADR-0009).
//!
//! Owns the **Project** aggregate: lifecycle status, primary location, prime contractor, and
//! client company participants. Worker access is **orchestrated** through Core
//! [`MembershipApi`](proven_core::MembershipApi) — this module never stores a competing ACL.
//!
//! Skeleton scope: create, update, archive, membership. No safety features, inspections, or
//! forms. See `domain::ownership` for deferred responsibilities (Equipment, Safety, Documents,
//! Settings APIs).

pub mod api;
pub mod application;
pub mod domain;
pub mod events;
pub mod infrastructure;

use std::sync::Arc;

use axum::Router;
use proven_core::{AuthzApi, MembershipApi, TenancyApi};

pub use application::services::ActingContext;
pub use application::{ProjectsApi, ProjectsPorts, ProjectsServices};
pub use domain::{
    ParticipantId, ParticipantStatus, ParticipationRole, Project, ProjectLocation, ProjectStatus,
    ProjectsError,
};
pub use events::ProjectsEvent;

/// Process-local handle to Projects: shared services + the HTTP router builder.
#[derive(Clone)]
pub struct ProjectsModule {
    pub services: Arc<ProjectsServices>,
}

impl ProjectsModule {
    /// In-memory ports + stub Allow-all AuthZ, no Core membership wiring.
    pub fn in_memory() -> Self {
        Self {
            services: Arc::new(ProjectsServices::in_memory_unchecked()),
        }
    }

    /// In-memory ports wired to real Core `AuthzApi` + `MembershipApi` + `TenancyApi`.
    pub fn with_core<C>(core: Arc<C>) -> Self
    where
        C: AuthzApi + MembershipApi + TenancyApi + Send + Sync + 'static,
    {
        Self {
            services: Arc::new(ProjectsServices::with_core(
                ProjectsPorts::in_memory(),
                core,
            )),
        }
    }

    pub fn from_ports(
        ports: ProjectsPorts,
        authz: Arc<dyn AuthzApi>,
        membership: Option<Arc<dyn MembershipApi>>,
        tenancy: Option<Arc<dyn TenancyApi>>,
    ) -> Self {
        Self {
            services: Arc::new(ProjectsServices::new(ports, authz, membership, tenancy)),
        }
    }

    /// Mount Projects' HTTP surface under `/api/v1/projects/*`.
    pub fn router(self) -> Router {
        api::router(self)
    }
}
