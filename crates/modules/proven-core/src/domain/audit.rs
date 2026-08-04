//! Audit Engine domain types (ADR-0008, AUDIT_LOGGING_ARCHITECTURE.md §4, §11). Pure value
//! types only — the engine itself lives in `application::services::audit_service`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use proven_shared::{CompanyId, ProjectId, TenantId, UserId};

/// A single field-level diff captured alongside an [`super::AuditEntry`] (§4 `before_ref` /
/// `after_ref` — expressed here as inline old/new pairs rather than version pointers, since Core
/// does not keep a generic version store).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditChange {
    pub field: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new: Option<serde_json::Value>,
}

/// Stable audit category facet (AUDIT_LOGGING_ARCHITECTURE.md §4, §6). Stored on
/// [`super::AuditEntry::category`] as its `snake_case` string — the enum exists for compile-time
/// safe construction; the SQL `CHECK` constraint is the source of truth for valid values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditCategory {
    Auth,
    Authz,
    Data,
    Signature,
    Workflow,
    Admin,
    Export,
    Other,
}

impl AuditCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auth => "auth",
            Self::Authz => "authz",
            Self::Data => "data",
            Self::Signature => "signature",
            Self::Workflow => "workflow",
            Self::Admin => "admin",
            Self::Export => "export",
            Self::Other => "other",
        }
    }
}

impl std::fmt::Display for AuditCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Outcome facet (AUDIT_LOGGING_ARCHITECTURE.md §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    Success,
    Deny,
    Failure,
}

impl AuditOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Deny => "deny",
            Self::Failure => "failure",
        }
    }
}

impl std::fmt::Display for AuditOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Retention tier driving [`crate::application::services::audit_service::AuditService::list_purge_candidates`]
/// (AUDIT_LOGGING_ARCHITECTURE.md §9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditRetentionClass {
    Standard,
    Security,
    Compliance,
    Restricted,
}

impl AuditRetentionClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Security => "security",
            Self::Compliance => "compliance",
            Self::Restricted => "restricted",
        }
    }
}

impl std::fmt::Display for AuditRetentionClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Filter set for [`crate::application::ports::AuditRepository::search`] and
/// `AuditService::search` (AUDIT_LOGGING_ARCHITECTURE.md §11.2). Every field is optional — an
/// empty query is an unfiltered, tenant-scoped listing (matches the legacy `query` behavior).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditSearchQuery {
    pub actor_user_id: Option<UserId>,
    /// Exact match on `action` today; prefix/wildcard search is a future enhancement.
    pub action: Option<String>,
    pub module_key: Option<String>,
    pub category: Option<String>,
    pub project_id: Option<ProjectId>,
    pub company_id: Option<CompanyId>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub workflow_instance_id: Option<Uuid>,
    pub signature_package_id: Option<Uuid>,
    pub outcome: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    /// Case-insensitive substring match against `action` or the stringified `payload`.
    pub q: Option<String>,
}

impl AuditSearchQuery {
    pub fn is_empty(&self) -> bool {
        self.actor_user_id.is_none()
            && self.action.is_none()
            && self.module_key.is_none()
            && self.category.is_none()
            && self.project_id.is_none()
            && self.company_id.is_none()
            && self.resource_type.is_none()
            && self.resource_id.is_none()
            && self.workflow_instance_id.is_none()
            && self.signature_package_id.is_none()
            && self.outcome.is_none()
            && self.from.is_none()
            && self.to.is_none()
            && self.q.is_none()
    }
}

/// Metadata for an async audit export request (ADR-0008 §3, `core.audit_export_jobs`). Bytes
/// live in object storage in a future iteration (ADR-0008 consequence); today this row records
/// job lifecycle, the filter used, and the resulting entry count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditExportJob {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub requested_by: Option<UserId>,
    /// `queued` | `running` | `completed` | `failed`.
    pub status: String,
    pub filter: serde_json::Value,
    pub entry_count: Option<i32>,
    pub storage_key: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Advisory per-tenant retention windows by [`super::AuditEntry::retention_class`]
/// (AUDIT_LOGGING_ARCHITECTURE.md §9, `core.audit_retention_policies`). Consulted by
/// `AuditService::list_purge_candidates` — never triggers deletion itself; audit facts are
/// append-only and archival/purge remains an ops-driven, out-of-band process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRetentionPolicy {
    pub tenant_id: TenantId,
    pub standard_days: i32,
    pub security_days: i32,
    pub compliance_days: i32,
    pub restricted_days: i32,
    pub export_before_purge: bool,
    pub updated_at: DateTime<Utc>,
}

impl AuditRetentionPolicy {
    /// Matches the column defaults in
    /// `db/migrations/core/20260803240000_core_audit_engine.sql` — returned when a tenant has no
    /// explicit override so `get_retention_policy` never has to error.
    pub fn default_for(tenant_id: TenantId) -> Self {
        Self {
            tenant_id,
            standard_days: 2555,
            security_days: 2555,
            compliance_days: 2555,
            restricted_days: 3650,
            export_before_purge: true,
            updated_at: Utc::now(),
        }
    }
}
