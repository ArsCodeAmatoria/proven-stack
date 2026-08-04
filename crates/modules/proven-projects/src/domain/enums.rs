//! Lifecycle and participation enums ([PROJECTS_DOMAIN.md](../../../../../docs/architecture/PROJECTS_DOMAIN.md) §7.2).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    Planning,
    Active,
    OnHold,
    Closed,
    Archived,
}

impl ProjectStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Planning => "planning",
            Self::Active => "active",
            Self::OnHold => "on_hold",
            Self::Closed => "closed",
            Self::Archived => "archived",
        }
    }

    pub fn is_archived(self) -> bool {
        matches!(self, Self::Archived)
    }

    /// Whether workers may still be assigned (skeleton gate).
    pub fn accepts_membership(self) -> bool {
        !matches!(self, Self::Closed | Self::Archived)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipationRole {
    Prime,
    Subcontractor,
    Client,
    Supplier,
    Other,
}

impl ParticipationRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Prime => "prime",
            Self::Subcontractor => "subcontractor",
            Self::Client => "client",
            Self::Supplier => "supplier",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantStatus {
    Invited,
    Active,
    Suspended,
    Removed,
}

impl ParticipantStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Invited => "invited",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Removed => "removed",
        }
    }
}
