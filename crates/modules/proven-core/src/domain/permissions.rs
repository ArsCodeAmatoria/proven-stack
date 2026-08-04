//! Stable Core permission codes — matches `db/migrations/core/*_core_permissions_seed.sql` and
//! `db/migrations/core/20260803230001_core_enterprise_rbac_seed.sql` (ADR-0007).
//!
//! Permission codes are append-only (see CORE_DOMAIN.md §21). Never rename in place.

use proven_shared::RoleId;
use uuid::Uuid;

pub const TENANT_READ: &str = "core.tenant.read";
pub const TENANT_MANAGE: &str = "core.tenant.manage";
pub const COMPANY_MANAGE: &str = "core.company.manage";
pub const ORG_MANAGE: &str = "core.org.manage";
pub const USER_INVITE: &str = "core.user.invite";
pub const USER_MANAGE: &str = "core.user.manage";
pub const ROLE_MANAGE: &str = "core.role.manage";
pub const GRANT_MANAGE: &str = "core.grant.manage";
pub const MEMBERSHIP_MANAGE: &str = "core.membership.manage";
pub const TEAM_MANAGE: &str = "core.team.manage";
pub const FILE_UPLOAD: &str = "core.file.upload";
pub const FILE_READ: &str = "core.file.read";
pub const FILE_DELETE: &str = "core.file.delete";
pub const AUDIT_READ: &str = "core.audit.read";
pub const AUDIT_EXPORT: &str = "core.audit.export";
pub const SETTINGS_MANAGE: &str = "core.settings.manage";
pub const FLAGS_MANAGE: &str = "core.flags.manage";
pub const LICENSE_READ: &str = "core.license.read";

// --- ADR-0007: enterprise RBAC additions to the Core-owned (`core.*`) catalog ---
pub const COMPANY_READ: &str = "core.company.read";
pub const ROLE_READ: &str = "core.role.read";
pub const GRANT_READ: &str = "core.grant.read";
pub const OVERRIDE_MANAGE: &str = "core.override.manage";

/// Full catalog of Core-owned (`core.*`) permission codes.
pub const ALL_CORE_PERMISSIONS: &[&str] = &[
    TENANT_READ,
    TENANT_MANAGE,
    COMPANY_MANAGE,
    ORG_MANAGE,
    USER_INVITE,
    USER_MANAGE,
    ROLE_MANAGE,
    GRANT_MANAGE,
    MEMBERSHIP_MANAGE,
    TEAM_MANAGE,
    FILE_UPLOAD,
    FILE_READ,
    FILE_DELETE,
    AUDIT_READ,
    AUDIT_EXPORT,
    SETTINGS_MANAGE,
    FLAGS_MANAGE,
    LICENSE_READ,
    COMPANY_READ,
    ROLE_READ,
    GRANT_READ,
    OVERRIDE_MANAGE,
];

// --- ADR-0007 §6 / AUTHORIZATION_RBAC_ARCHITECTURE.md: module permission catalog. ---
//
// Modules **propose** these codes; Core **publishes** the catalog (documents/equipment/
// training/safety/approvals/projects/feature are not implemented modules yet — only their
// permission codes + gating prefixes exist today, per the task constraint "do not implement
// Projects/Documents/Safety modules — only permission catalog + engines").

pub const FEATURE_MODULE_ACCESS: &str = "feature.module.access";
pub const FEATURE_FLAG_EVALUATE: &str = "feature.flag.evaluate";
pub const FEATURE_PERMISSIONS: &[&str] = &[FEATURE_MODULE_ACCESS, FEATURE_FLAG_EVALUATE];

pub const DOCUMENTS_DOCUMENT_READ: &str = "documents.document.read";
pub const DOCUMENTS_DOCUMENT_MANAGE: &str = "documents.document.manage";
pub const DOCUMENTS_VERSION_PUBLISH: &str = "documents.version.publish";
pub const DOCUMENTS_ACK_MANAGE: &str = "documents.ack.manage";
pub const DOCUMENTS_ACL_MANAGE: &str = "documents.acl.manage";
pub const DOCUMENTS_PERMISSIONS: &[&str] = &[
    DOCUMENTS_DOCUMENT_READ,
    DOCUMENTS_DOCUMENT_MANAGE,
    DOCUMENTS_VERSION_PUBLISH,
    DOCUMENTS_ACK_MANAGE,
    DOCUMENTS_ACL_MANAGE,
];

pub const APPROVALS_REQUEST_CREATE: &str = "approvals.request.create";
pub const APPROVALS_REQUEST_APPROVE: &str = "approvals.request.approve";
pub const APPROVALS_REQUEST_REJECT: &str = "approvals.request.reject";
pub const APPROVALS_POLICY_MANAGE: &str = "approvals.policy.manage";
pub const APPROVALS_PERMISSIONS: &[&str] = &[
    APPROVALS_REQUEST_CREATE,
    APPROVALS_REQUEST_APPROVE,
    APPROVALS_REQUEST_REJECT,
    APPROVALS_POLICY_MANAGE,
];

pub const EQUIPMENT_ASSET_READ: &str = "equipment.asset.read";
pub const EQUIPMENT_ASSET_MANAGE: &str = "equipment.asset.manage";
pub const EQUIPMENT_INSPECTION_PERFORM: &str = "equipment.inspection.perform";
pub const EQUIPMENT_READINESS_OVERRIDE: &str = "equipment.readiness.override";
pub const EQUIPMENT_PERMISSIONS: &[&str] = &[
    EQUIPMENT_ASSET_READ,
    EQUIPMENT_ASSET_MANAGE,
    EQUIPMENT_INSPECTION_PERFORM,
    EQUIPMENT_READINESS_OVERRIDE,
];

pub const TRAINING_COURSE_READ: &str = "training.course.read";
pub const TRAINING_COURSE_MANAGE: &str = "training.course.manage";
pub const TRAINING_ASSIGNMENT_MANAGE: &str = "training.assignment.manage";
pub const TRAINING_COMPLETION_RECORD: &str = "training.completion.record";
pub const TRAINING_PERMISSIONS: &[&str] = &[
    TRAINING_COURSE_READ,
    TRAINING_COURSE_MANAGE,
    TRAINING_ASSIGNMENT_MANAGE,
    TRAINING_COMPLETION_RECORD,
];

pub const SAFETY_ACTIVITY_CREATE: &str = "safety.activity.create";
pub const SAFETY_ACTIVITY_SUBMIT: &str = "safety.activity.submit";
pub const SAFETY_ACTIVITY_REVIEW: &str = "safety.activity.review";
pub const SAFETY_INCIDENT_MANAGE: &str = "safety.incident.manage";
pub const SAFETY_CA_MANAGE: &str = "safety.ca.manage";
pub const SAFETY_PERMISSIONS: &[&str] = &[
    SAFETY_ACTIVITY_CREATE,
    SAFETY_ACTIVITY_SUBMIT,
    SAFETY_ACTIVITY_REVIEW,
    SAFETY_INCIDENT_MANAGE,
    SAFETY_CA_MANAGE,
];

pub const PROJECTS_PROJECT_READ: &str = "projects.project.read";
pub const PROJECTS_PROJECT_MANAGE: &str = "projects.project.manage";
pub const PROJECTS_PROJECT_CREATE: &str = "projects.project.create";
pub const PROJECTS_PERMISSIONS: &[&str] = &[
    PROJECTS_PROJECT_READ,
    PROJECTS_PROJECT_MANAGE,
    PROJECTS_PROJECT_CREATE,
];

/// Every non-Core module permission code published so far — handy for seeding / admin catalog
/// browsing. Grows as modules propose more codes (append-only).
pub const ALL_MODULE_PERMISSION_SAMPLES: &[&str] = &[
    FEATURE_MODULE_ACCESS,
    FEATURE_FLAG_EVALUATE,
    DOCUMENTS_DOCUMENT_READ,
    DOCUMENTS_DOCUMENT_MANAGE,
    DOCUMENTS_VERSION_PUBLISH,
    DOCUMENTS_ACK_MANAGE,
    DOCUMENTS_ACL_MANAGE,
    APPROVALS_REQUEST_CREATE,
    APPROVALS_REQUEST_APPROVE,
    APPROVALS_REQUEST_REJECT,
    APPROVALS_POLICY_MANAGE,
    EQUIPMENT_ASSET_READ,
    EQUIPMENT_ASSET_MANAGE,
    EQUIPMENT_INSPECTION_PERFORM,
    EQUIPMENT_READINESS_OVERRIDE,
    TRAINING_COURSE_READ,
    TRAINING_COURSE_MANAGE,
    TRAINING_ASSIGNMENT_MANAGE,
    TRAINING_COMPLETION_RECORD,
    SAFETY_ACTIVITY_CREATE,
    SAFETY_ACTIVITY_SUBMIT,
    SAFETY_ACTIVITY_REVIEW,
    SAFETY_INCIDENT_MANAGE,
    SAFETY_CA_MANAGE,
    PROJECTS_PROJECT_READ,
    PROJECTS_PROJECT_MANAGE,
    PROJECTS_PROJECT_CREATE,
];

/// Module key prefixes gated by [`crate::application::LicenseApi::is_module_enabled`] before
/// RBAC is evaluated (AUTHORIZATION_RBAC_ARCHITECTURE.md §8). Permission codes outside this list
/// (e.g. `core.*`) are never license-gated — Core itself is foundational.
pub const LICENSE_GATED_MODULE_PREFIXES: &[&str] = &[
    "documents",
    "equipment",
    "training",
    "safety",
    "approvals",
    "projects",
    "feature",
];

/// System Tenant Admin role UUID — matches the SQL seed
/// (`db/migrations/core/20260803200001_core_permissions_seed.sql`).
pub const SYSTEM_TENANT_ADMIN_ROLE_UUID: Uuid =
    Uuid::from_u128(0x0000_0000_0000_4000_8000_0000_0000_0001);

pub fn system_tenant_admin_role_id() -> RoleId {
    RoleId(SYSTEM_TENANT_ADMIN_ROLE_UUID)
}

// --- ADR-0007 §5 system role UUIDs — matches
// `db/migrations/core/20260803230001_core_enterprise_rbac_seed.sql`. ---

pub const SYSTEM_COMPANY_ADMIN_ROLE_UUID: Uuid =
    Uuid::from_u128(0x0000_0000_0000_4000_8000_0000_0000_0010);
pub const SYSTEM_PROJECT_ADMIN_ROLE_UUID: Uuid =
    Uuid::from_u128(0x0000_0000_0000_4000_8000_0000_0000_0011);
pub const SYSTEM_SUPERVISOR_ROLE_UUID: Uuid =
    Uuid::from_u128(0x0000_0000_0000_4000_8000_0000_0000_0012);
pub const SYSTEM_WORKER_ROLE_UUID: Uuid =
    Uuid::from_u128(0x0000_0000_0000_4000_8000_0000_0000_0013);
pub const SYSTEM_SAFETY_COORDINATOR_ROLE_UUID: Uuid =
    Uuid::from_u128(0x0000_0000_0000_4000_8000_0000_0000_0014);
pub const SYSTEM_EQUIPMENT_MANAGER_ROLE_UUID: Uuid =
    Uuid::from_u128(0x0000_0000_0000_4000_8000_0000_0000_0015);
pub const SYSTEM_TRAINING_ADMIN_ROLE_UUID: Uuid =
    Uuid::from_u128(0x0000_0000_0000_4000_8000_0000_0000_0016);
pub const SYSTEM_DOCUMENT_CONTROL_ROLE_UUID: Uuid =
    Uuid::from_u128(0x0000_0000_0000_4000_8000_0000_0000_0017);
pub const SYSTEM_TEMPORARY_ELEVATED_ROLE_UUID: Uuid =
    Uuid::from_u128(0x0000_0000_0000_4000_8000_0000_0000_0018);

pub fn company_admin_role_id() -> RoleId {
    RoleId(SYSTEM_COMPANY_ADMIN_ROLE_UUID)
}

pub fn project_admin_role_id() -> RoleId {
    RoleId(SYSTEM_PROJECT_ADMIN_ROLE_UUID)
}

pub fn supervisor_role_id() -> RoleId {
    RoleId(SYSTEM_SUPERVISOR_ROLE_UUID)
}

pub fn worker_role_id() -> RoleId {
    RoleId(SYSTEM_WORKER_ROLE_UUID)
}

pub fn safety_coordinator_role_id() -> RoleId {
    RoleId(SYSTEM_SAFETY_COORDINATOR_ROLE_UUID)
}

pub fn equipment_manager_role_id() -> RoleId {
    RoleId(SYSTEM_EQUIPMENT_MANAGER_ROLE_UUID)
}

pub fn training_admin_role_id() -> RoleId {
    RoleId(SYSTEM_TRAINING_ADMIN_ROLE_UUID)
}

pub fn document_control_role_id() -> RoleId {
    RoleId(SYSTEM_DOCUMENT_CONTROL_ROLE_UUID)
}

pub fn temporary_elevated_role_id() -> RoleId {
    RoleId(SYSTEM_TEMPORARY_ELEVATED_ROLE_UUID)
}
