//! Core enumerations / states — mirrors `db/migrations/core` CHECK constraints.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantStatus {
    Active,
    Suspended,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    Invited,
    Active,
    Locked,
    Deactivated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompanyType {
    Prime,
    Subcontractor,
    Crane,
    Forming,
    Civil,
    Industrial,
    Other,
}

/// Boundary for an [`crate::domain::AccessScope`]: Tenant, OrgUnit, Company, Project, Team, or
/// Self (ADR-0007 §4 adds `Company`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrantScopeType {
    Tenant,
    OrgUnit,
    Company,
    Project,
    Team,
    /// A principal's own user record (`self` in CORE_DOMAIN.md — `Self` is a Rust keyword).
    #[serde(rename = "self")]
    SelfScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MembershipStatus {
    Invited,
    Active,
    Suspended,
    Removed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileObjectStatus {
    PendingUpload,
    /// Upload complete; virus-scan / media pipeline in progress (ADR-0010).
    Processing,
    Available,
    Quarantined,
    Deleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LicenseStatus {
    Trial,
    Active,
    Grace,
    Expired,
    Suspended,
}

/// Role classification (ADR-0007 §3 expands system-role kinds beyond `System`).
///
/// `Company`, `Project`, and `Temporary` are still platform-shipped ("system") role kinds —
/// they simply describe the intended scope/lifecycle of the role, mirroring
/// `db/migrations/core/20260803230001_core_enterprise_rbac_seed.sql`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoleKind {
    System,
    TenantCustom,
    Membership,
    Company,
    Project,
    Temporary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrantKind {
    Standard,
    Delegation,
    Temporary,
    BreakGlass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompanyStatus {
    Active,
    Deactivated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrgUnitStatus {
    Active,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Active,
    Revoked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoleStatus {
    Active,
    Retired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamStatus {
    Active,
    Archived,
}

/// Precedence chain for [`crate::application::SettingsApi`]: User → OrgUnit → Tenant → Platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettingScopeType {
    Platform,
    Tenant,
    OrgUnit,
    User,
}

/// Effect of a [`crate::domain::models::PermissionOverride`] — `Deny` always wins over `Allow`
/// and over any covering role grant (ADR-0007 §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverrideEffect {
    Allow,
    Deny,
}

/// Grouping used for permission catalog browsing / policy authoring — mirrors
/// `core.permissions.family` (ADR-0007 §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionFamily {
    Core,
    Feature,
    Documents,
    Approvals,
    Equipment,
    Training,
    Safety,
    Projects,
    Other,
}

/// Sensitivity classification driving step-up / audit emphasis — mirrors
/// `core.permissions.sensitivity` (ADR-0007 §6, AUTHORIZATION_RBAC_ARCHITECTURE.md §6.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionSensitivity {
    Standard,
    Elevated,
    BreakGlass,
}
