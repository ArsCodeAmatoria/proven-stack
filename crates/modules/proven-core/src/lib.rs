//! Proven Core — the platform foundation module (CORE_DOMAIN.md, ADR-0001..0004).
//!
//! Owns tenancy, identity, AuthZ, project membership, teams, file object metadata, audit,
//! settings, feature flags, and licensing. This is the **only** domain module implemented in
//! the foundation milestone; see `crates/modules/README.md`.
//!
//! Other modules and Temporal activities must depend only on the public interfaces re-exported
//! here (`TenancyApi`, `IdentityApi`, `AuthzApi`, `MembershipApi`, `FileApi`, `AuditApi`,
//! `SettingsApi`, `FlagsApi`, `LicenseApi`) — never on `domain`/`infrastructure` internals or
//! Core's Postgres schema directly (ADR-0001, ADR-0003).

pub mod api;
pub mod application;
pub mod domain;
pub mod events;
pub mod infrastructure;

use std::sync::Arc;

use axum::Router;

pub use application::{
    AuditApi, AuthzApi, CorePorts, CoreServices, FileApi, FlagsApi, IdentityApi, LicenseApi,
    MembershipApi, SettingsApi, TenancyApi,
};
pub use infrastructure::{
    EnqueuePendingVirusScanHook, PassthroughVirusScanHook, PendingR2ObjectStorage,
    PlaceholderObjectStorage, R2StorageConfig,
};
pub use domain::{
    AccessGrant, AccessScope, AuditCategory, AuditChange, AuditEntry, AuditExportJob,
    AuditOutcome, AuditRetentionClass, AuditRetentionPolicy, AuditSearchQuery, AuthzDecision,
    Company, CompanyStatus, CompanyType, CoreError, DownloadLink, FeatureFlag, FileLinkKind,
    FileObject, FileObjectClass, FileObjectStatus, FileShareLink, GrantKind, GrantScopeType,
    License, LicenseStatus, MembershipStatus, ModuleEntitlement, OrgUnit, OrgUnitStatus,
    PresignedUrl, ProjectMembership, RoleDefinition, RoleKind, RoleStatus, Session, SessionStatus,
    SettingEntry, SettingScopeType, Team, TeamMember, TeamStatus, Tenant, TenantStatus, UploadIntent,
    User, UserStatus, VirusScanOutcome, VirusScanStatus,
};
pub use events::CoreEvent;

/// Process-local handle to Core: shared services + the HTTP router builder.
///
/// ```
/// use proven_core::CoreModule;
///
/// let module = CoreModule::in_memory();
/// let _router = module.router();
/// ```
#[derive(Clone)]
pub struct CoreModule {
    pub services: Arc<CoreServices>,
}

impl CoreModule {
    /// Build a Core module backed entirely by the seeded in-memory store — no Postgres
    /// required. Used for unit tests and no-DB local development.
    pub fn in_memory() -> Self {
        Self {
            services: Arc::new(CoreServices::in_memory()),
        }
    }

    /// Build a Core module from an arbitrary set of repository ports (e.g. Postgres adapters).
    pub fn from_ports(ports: CorePorts) -> Self {
        Self {
            services: Arc::new(CoreServices::new(ports)),
        }
    }

    /// Mount Core's HTTP surface under `/api/v1/core/*`. Callers merge this into the platform
    /// host router (CORE_DOMAIN.md §13.2).
    pub fn router(self) -> Router {
        api::router(self)
    }
}
