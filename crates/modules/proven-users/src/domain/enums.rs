//! Users enumerations / states — mirrors the CHECK constraints in
//! `db/migrations/users/20260803220000_users_schema.sql` (ADR-0006).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileStatus {
    Active,
    Archived,
}

/// Profile-level classification tag for UX/directory purposes. **Not** Core RBAC
/// (`proven_core::domain::RoleDefinition`/`AccessGrant`) and **not** a People workforce role — see
/// `domain::ownership`. AuthZ decisions never branch on `UserKind`, only on `AuthzApi`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserKind {
    Worker,
    Supervisor,
    Manager,
    SafetyCoordinator,
    Administrator,
    External,
    Guest,
}

impl UserKind {
    /// Stable wire/storage token, matching the SQL `CHECK` constraint values exactly.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Worker => "worker",
            Self::Supervisor => "supervisor",
            Self::Manager => "manager",
            Self::SafetyCoordinator => "safety_coordinator",
            Self::Administrator => "administrator",
            Self::External => "external",
            Self::Guest => "guest",
        }
    }
}

/// How often a user's rolled-up notification digest is sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DigestCadence {
    Realtime,
    Hourly,
    Daily,
    Weekly,
    Off,
}

/// A user's preferred signing mechanism (signing *preference*, not a signature package — those
/// are owned by the Signatures module, see `domain::ownership`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignatureType {
    Drawn,
    Typed,
    Uploaded,
    Clickwrap,
}
