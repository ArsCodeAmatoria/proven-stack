//! Core domain layer — pure types and rules, no I/O (CORE_DOMAIN.md §5-§12, ADR-0007..0010).

mod audit;
mod authz;
mod enums;
mod error;
pub mod files;
mod models;
pub mod permissions;
pub mod rbac;

pub use audit::{
    AuditCategory, AuditChange, AuditExportJob, AuditOutcome, AuditRetentionClass,
    AuditRetentionPolicy, AuditSearchQuery,
};
pub use authz::{AccessScope, AuthzDecision};
pub use enums::{
    CompanyStatus, CompanyType, FileObjectStatus, GrantKind, GrantScopeType, LicenseStatus,
    MembershipStatus, OrgUnitStatus, OverrideEffect, PermissionFamily, PermissionSensitivity,
    RoleKind, RoleStatus, SessionStatus, SettingScopeType, TeamStatus, TenantStatus, UserStatus,
};
pub use error::CoreError;
pub use files::{
    DownloadLink, FileLinkKind, FileObjectClass, FileShareLink, PresignedUrl, UploadIntent,
    VirusScanOutcome, VirusScanRequest, VirusScanStatus,
};
pub use models::{
    AccessGrant, AuditEntry, Company, FeatureFlag, FileObject, License, ModuleEntitlement, OrgUnit,
    PermissionOverride, ProjectMembership, RoleDefinition, Session, SettingEntry, Team, TeamMember,
    Tenant, User,
};
pub use rbac::{
    AbacContext, AuthorizationPolicy, DefaultRbacPolicy, EvaluationInput, PermissionEngine,
    RoleEngine, SealedResourcePolicy,
};
