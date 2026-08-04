//! Authorization decision types (ADR-0003: `AuthzApi` is the only decision authority).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::enums::GrantScopeType;

/// A concrete boundary: either a grant's boundary or a resource's boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessScope {
    pub scope_type: GrantScopeType,
    pub scope_id: Option<Uuid>,
}

impl AccessScope {
    pub fn tenant() -> Self {
        Self {
            scope_type: GrantScopeType::Tenant,
            scope_id: None,
        }
    }

    pub fn company(company_id: Uuid) -> Self {
        Self {
            scope_type: GrantScopeType::Company,
            scope_id: Some(company_id),
        }
    }

    pub fn project(project_id: Uuid) -> Self {
        Self {
            scope_type: GrantScopeType::Project,
            scope_id: Some(project_id),
        }
    }

    pub fn self_scope(user_id: Uuid) -> Self {
        Self {
            scope_type: GrantScopeType::SelfScope,
            scope_id: Some(user_id),
        }
    }
}

/// Result of `Principal + PermissionCode + ResourceScope → Allow/Deny`.
///
/// Fail-closed: absence of an explicit `Allow` is a `Deny`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum AuthzDecision {
    Allow { reasons: Vec<String> },
    Deny { reasons: Vec<String> },
}

impl AuthzDecision {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allow { .. })
    }

    pub fn deny(reason: impl Into<String>) -> Self {
        Self::Deny {
            reasons: vec![reason.into()],
        }
    }

    pub fn allow(reason: impl Into<String>) -> Self {
        Self::Allow {
            reasons: vec![reason.into()],
        }
    }
}
