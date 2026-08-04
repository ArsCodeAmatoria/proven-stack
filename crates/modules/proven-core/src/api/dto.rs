//! HTTP request/response DTOs. Domain models already derive `Serialize`/`Deserialize` and
//! contain no secrets, so they are returned directly; this module holds request bodies and
//! the few response envelopes that combine multiple aggregates.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{Company, CompanyType, GrantKind, GrantScopeType, License, OverrideEffect, Tenant, User};

#[derive(Debug, Deserialize)]
pub struct ProvisionTenantRequest {
    pub slug: String,
    pub display_name: String,
    pub region_code: String,
    pub owner_company_name: String,
    #[serde(default)]
    pub owner_company_type: Option<CompanyType>,
    pub admin_email: String,
    pub admin_display_name: String,
    #[serde(default)]
    pub seats_limit: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ProvisionTenantResponse {
    pub tenant: Tenant,
    pub owner_company: Company,
    pub admin_user: User,
    pub license: License,
}

#[derive(Debug, Deserialize)]
pub struct RegisterCompanyRequest {
    pub tenant_id: Uuid,
    pub legal_name: String,
    pub display_name: String,
    pub company_type: CompanyType,
}

#[derive(Debug, Deserialize)]
pub struct InviteUserRequest {
    pub email: String,
    pub display_name: String,
}

#[derive(Debug, Deserialize)]
pub struct GrantAccessRequest {
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub scope_type: GrantScopeType,
    #[serde(default)]
    pub scope_id: Option<Uuid>,
    #[serde(default)]
    pub grant_kind: Option<GrantKind>,
    #[serde(default)]
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// ABAC inputs on the wire (ADR-0007 §8) — optional; missing fields mean "no signal".
#[derive(Debug, Default, Deserialize)]
pub struct AbacHttpContext {
    #[serde(default)]
    pub resource_attributes: HashMap<String, String>,
    #[serde(default)]
    pub assurance_level: Option<String>,
    #[serde(default)]
    pub resource_state: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AuthorizeHttpRequest {
    pub tenant_id: Uuid,
    pub principal_id: Uuid,
    pub permission: String,
    pub scope_type: GrantScopeType,
    #[serde(default)]
    pub scope_id: Option<Uuid>,
    #[serde(default)]
    pub abac: AbacHttpContext,
}

#[derive(Debug, Deserialize)]
pub struct UpsertPermissionOverrideRequest {
    pub user_id: Uuid,
    pub permission: String,
    pub effect: OverrideEffect,
    pub scope_type: GrantScopeType,
    #[serde(default)]
    pub scope_id: Option<Uuid>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct ListPermissionOverridesQuery {
    pub user_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct GrantProjectMembershipRequest {
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub membership_role: String,
}

#[derive(Debug, Serialize)]
pub struct IsProjectMemberResponse {
    pub project_id: Uuid,
    pub is_member: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateTeamRequest {
    pub name: String,
    #[serde(default)]
    pub project_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct AppendAuditRequest {
    pub action: String,
    pub resource_type: String,
    #[serde(default)]
    pub resource_id: Option<Uuid>,
    #[serde(default)]
    pub correlation_id: Option<Uuid>,
    #[serde(default)]
    pub causation_id: Option<Uuid>,
    #[serde(default = "default_payload")]
    pub payload: serde_json::Value,
    #[serde(default)]
    pub module_key: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub outcome: Option<String>,
    #[serde(default)]
    pub project_id: Option<Uuid>,
    #[serde(default)]
    pub company_id: Option<Uuid>,
    #[serde(default)]
    pub old_value: Option<serde_json::Value>,
    #[serde(default)]
    pub new_value: Option<serde_json::Value>,
}

fn default_payload() -> serde_json::Value {
    serde_json::json!({})
}

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub offset: Option<u32>,
}

/// `GET /api/v1/core/audit` query params for [`crate::api::handlers::search_audit`]
/// (AUDIT_LOGGING_ARCHITECTURE.md §11.2) — mirrors
/// [`crate::domain::AuditSearchQuery`] with wire-friendly `Uuid`s in place of typed ids.
#[derive(Debug, Deserialize)]
pub struct AuditSearchHttpQuery {
    #[serde(default)]
    pub actor_user_id: Option<Uuid>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub module_key: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub project_id: Option<Uuid>,
    #[serde(default)]
    pub company_id: Option<Uuid>,
    #[serde(default)]
    pub resource_type: Option<String>,
    #[serde(default)]
    pub resource_id: Option<Uuid>,
    #[serde(default)]
    pub workflow_instance_id: Option<Uuid>,
    #[serde(default)]
    pub signature_package_id: Option<Uuid>,
    #[serde(default)]
    pub outcome: Option<String>,
    #[serde(default)]
    pub from: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub to: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub offset: Option<u32>,
}

/// `POST /api/v1/core/audit/exports` body — same filter shape as [`AuditSearchHttpQuery`] minus
/// paging (an export always collects every matching entry).
#[derive(Debug, Default, Deserialize)]
pub struct RequestAuditExportRequest {
    #[serde(default)]
    pub actor_user_id: Option<Uuid>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub module_key: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub project_id: Option<Uuid>,
    #[serde(default)]
    pub company_id: Option<Uuid>,
    #[serde(default)]
    pub resource_type: Option<String>,
    #[serde(default)]
    pub resource_id: Option<Uuid>,
    #[serde(default)]
    pub workflow_instance_id: Option<Uuid>,
    #[serde(default)]
    pub signature_package_id: Option<Uuid>,
    #[serde(default)]
    pub outcome: Option<String>,
    #[serde(default)]
    pub from: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub to: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub q: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertAuditRetentionPolicyRequest {
    pub standard_days: i32,
    pub security_days: i32,
    pub compliance_days: i32,
    pub restricted_days: i32,
    #[serde(default = "default_export_before_purge")]
    pub export_before_purge: bool,
}

fn default_export_before_purge() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct UpsertSettingRequest {
    pub scope_type: crate::domain::SettingScopeType,
    #[serde(default)]
    pub scope_id: Option<Uuid>,
    pub key: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct GetSettingQuery {
    pub scope_type: crate::domain::SettingScopeType,
    #[serde(default)]
    pub scope_id: Option<Uuid>,
    pub key: String,
}

#[derive(Debug, Deserialize)]
pub struct EvaluateFlagRequest {
    pub key: String,
    #[serde(default)]
    pub tenant_id: Option<Uuid>,
    #[serde(default)]
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct EvaluateFlagResponse {
    pub key: String,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateFileUploadIntentRequest {
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub retention_class: Option<String>,
    #[serde(default)]
    pub access_class: Option<String>,
    #[serde(default)]
    pub object_class: Option<String>,
    #[serde(default)]
    pub original_filename: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub parent_file_id: Option<Uuid>,
    #[serde(default)]
    pub is_temporary: bool,
}

#[derive(Debug, Deserialize)]
pub struct CompleteFileUploadRequest {
    pub checksum_sha256: String,
    pub byte_size: i64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFileMetadataRequest {
    pub metadata: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct CreatePublicShareLinkRequest {
    #[serde(default)]
    pub ttl_hours: Option<i64>,
    #[serde(default)]
    pub max_downloads: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ApplyScanResultRequest {
    /// One of: clean | infected | pending | error
    pub outcome: String,
    #[serde(default)]
    pub detail: Option<String>,
}
