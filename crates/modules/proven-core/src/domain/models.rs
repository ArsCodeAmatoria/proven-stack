//! Core aggregates, entities, and value objects (CORE_DOMAIN.md §5-§7).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use proven_shared::{
    AuditEntryId, CausationId, CompanyId, CorrelationId, FeatureFlagKey, FileObjectId, GrantId,
    LicenseId, ModuleKey, OrgUnitId, PermissionCode, PermissionOverrideId, PersonId, ProjectId,
    ProjectMembershipId, RegionCode, RoleId, SessionId, SettingKey, TeamId, TenantId, UserId,
};

use super::enums::{
    CompanyStatus, CompanyType, FileObjectStatus, GrantKind, LicenseStatus, MembershipStatus,
    OrgUnitStatus, OverrideEffect, RoleKind, RoleStatus, SessionStatus, SettingScopeType,
    TeamStatus, TenantStatus, UserStatus,
};
use super::AccessScope;

/// Workspace lifecycle, region defaults, status, isolation root.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: TenantId,
    pub slug: String,
    pub display_name: String,
    pub region_code: RegionCode,
    pub status: TenantStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

/// Legal/operating company known to a tenant (owner company or partner/sub).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    pub id: CompanyId,
    pub tenant_id: TenantId,
    pub legal_name: String,
    pub display_name: String,
    pub company_type: CompanyType,
    pub status: CompanyStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

/// Hierarchical unit tree node within a tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgUnit {
    pub id: OrgUnitId,
    pub tenant_id: TenantId,
    pub parent_id: Option<OrgUnitId>,
    pub name: String,
    pub status: OrgUnitStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

/// Human account identity that can authenticate (AuthN UX lives in Better Auth; see ADR-0002).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub tenant_id: TenantId,
    pub email: String,
    pub display_name: String,
    pub status: UserStatus,
    /// Reference only — authority lives in Workforce.
    pub person_id: Option<PersonId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

/// Authenticatable session lifecycle and revocation ledger entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub status: SessionStatus,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Role name + permission set (system role when `tenant_id` is `None`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleDefinition {
    pub id: RoleId,
    pub tenant_id: Option<TenantId>,
    pub name: String,
    pub kind: RoleKind,
    pub status: RoleStatus,
    pub permissions: Vec<PermissionCode>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

impl RoleDefinition {
    pub fn has_permission(&self, code: &PermissionCode) -> bool {
        self.status == RoleStatus::Active && self.permissions.contains(code)
    }
}

/// Principal ↔ Role ↔ Scope binding (ADR-0003: the only authorization primitive).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessGrant {
    pub id: GrantId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub role_id: RoleId,
    pub scope: AccessScope,
    pub grant_kind: GrantKind,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<UserId>,
}

impl AccessGrant {
    pub fn is_active(&self, now: DateTime<Utc>) -> bool {
        self.revoked_at.is_none() && self.expires_at.map(|exp| exp > now).unwrap_or(true)
    }
}

/// Principal/Person ↔ Project participation and membership role binding.
///
/// Project **lifecycle** is owned by `projects`; this is only the access binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMembership {
    pub id: ProjectMembershipId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub user_id: Option<UserId>,
    pub person_id: Option<PersonId>,
    pub membership_role: String,
    pub status: MembershipStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

/// Named group of people for operational assignment (tenant- or project-scoped).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: TeamId,
    pub tenant_id: TenantId,
    pub name: String,
    pub project_id: Option<ProjectId>,
    pub status: TeamStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub team_id: TeamId,
    pub user_id: Option<UserId>,
    pub person_id: Option<PersonId>,
    pub added_at: DateTime<Utc>,
}

/// Stored binary with metadata and access policy hooks (not a controlled document).
/// Bytes live in R2; this row is the identity/AuthZ SoR (ADR-0010).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileObject {
    pub id: FileObjectId,
    pub tenant_id: TenantId,
    pub status: FileObjectStatus,
    pub storage_key: String,
    pub content_type: Option<String>,
    pub byte_size: Option<i64>,
    pub checksum_sha256: Option<String>,
    pub retention_class: String,
    pub access_class: String,
    pub created_by: Option<UserId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Optimistic concurrency / row revision.
    pub version: i64,
    /// Content class (photo, pdf, …) — drives R2 prefix + MIME allowlist.
    #[serde(default)]
    pub object_class: crate::domain::files::FileObjectClass,
    #[serde(default)]
    pub original_filename: Option<String>,
    /// Extensible metadata bag (EXIF policy flags, source, sensitivity, …).
    #[serde(default = "default_file_metadata")]
    pub metadata: serde_json::Value,
    /// Parent object when this row is a content version / derivative.
    #[serde(default)]
    pub parent_file_id: Option<FileObjectId>,
    /// Monotonic content version within a lineage (1 = original).
    #[serde(default = "default_content_version")]
    pub content_version: i32,
    #[serde(default)]
    pub is_temporary: bool,
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub scan_status: crate::domain::files::VirusScanStatus,
    #[serde(default)]
    pub scan_detail: Option<String>,
}

fn default_file_metadata() -> serde_json::Value {
    serde_json::json!({})
}

fn default_content_version() -> i32 {
    1
}

/// Append-only record of a significant action (CORE_DOMAIN.md §18, ADR-0008,
/// AUDIT_LOGGING_ARCHITECTURE.md §4). Facts are never updated or deleted after append —
/// corrections are new entries referencing the original by `resource_id`/`correlation_id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: AuditEntryId,
    pub tenant_id: TenantId,
    pub occurred_at: DateTime<Utc>,
    /// Server insert time — may lag `occurred_at` slightly for outbox-relayed appends.
    pub recorded_at: DateTime<Utc>,
    pub actor_user_id: Option<UserId>,
    pub actor_type: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub correlation_id: Option<CorrelationId>,
    pub causation_id: Option<CausationId>,
    pub payload: serde_json::Value,
    /// sha256 hex digest of `payload`, computed at append time — never mutated after.
    pub payload_digest: String,
    /// Owning module key (`safety`, `documents`, `core`, …) — enables per-module audit search.
    pub module_key: Option<String>,
    /// `auth` | `authz` | `data` | `signature` | `workflow` | `admin` | `export` | `other`.
    pub category: String,
    /// `success` | `deny` | `failure`.
    pub outcome: String,
    pub project_id: Option<ProjectId>,
    pub company_id: Option<CompanyId>,
    pub session_id: Option<SessionId>,
    pub ip_address: Option<String>,
    pub device_id: Option<String>,
    pub user_agent: Option<String>,
    pub workflow_instance_id: Option<Uuid>,
    pub signature_package_id: Option<Uuid>,
    /// Redacted pre-state snapshot — never raw secrets (hard rule: no secrets in payloads).
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    /// JSON array of [`super::AuditChange`] field diffs; defaults to `[]`.
    pub changes: serde_json::Value,
    /// `standard` | `security` | `compliance` | `restricted`.
    pub retention_class: String,
    /// `standard` | `restricted`.
    pub sensitivity: String,
    /// Hash-chain tamper-evidence tier (optional) — previous entry's `integrity_hash` for this
    /// tenant at append time.
    pub integrity_prev_hash: Option<String>,
    /// `sha256(integrity_prev_hash + payload_digest + action + id)` — computed at append time.
    pub integrity_hash: Option<String>,
}

/// Key/value entry within a tenant/org/user settings scope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingEntry {
    pub tenant_id: TenantId,
    pub scope_type: SettingScopeType,
    pub scope_id: Option<Uuid>,
    pub key: SettingKey,
    pub value: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

/// Runtime capability toggle (global/tenant/actor).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    pub key: FeatureFlagKey,
    pub description: String,
    pub default_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Commercial entitlement governing modules, seats, and limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub id: LicenseId,
    pub tenant_id: TenantId,
    pub status: LicenseStatus,
    pub plan_code: String,
    pub seats_limit: i32,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

impl License {
    /// Whether the license is in a state that permits platform writes.
    pub fn is_usable(&self) -> bool {
        matches!(
            self.status,
            LicenseStatus::Trial | LicenseStatus::Active | LicenseStatus::Grace
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleEntitlement {
    pub license_id: LicenseId,
    pub module_key: ModuleKey,
    pub enabled: bool,
}

/// Explicit allow/deny override for a single principal, evaluated by [`super::rbac::PermissionEngine`]
/// after grants (deny wins) — ADR-0007 §5, `core.permission_overrides`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionOverride {
    pub id: PermissionOverrideId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub permission_code: PermissionCode,
    pub effect: OverrideEffect,
    pub scope: AccessScope,
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<UserId>,
}

impl PermissionOverride {
    /// Not revoked and (no expiry or expiry still in the future) — same shape as
    /// [`AccessGrant::is_active`].
    pub fn is_active(&self, now: DateTime<Utc>) -> bool {
        self.revoked_at.is_none() && self.expires_at.map(|exp| exp > now).unwrap_or(true)
    }
}
