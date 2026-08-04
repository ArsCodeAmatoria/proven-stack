//! Typed identifiers shared across modules (no mutable aggregates).

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

macro_rules! id_newtype {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn from_uuid(id: Uuid) -> Self {
                Self(id)
            }

            pub fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<Uuid> for $name {
            fn from(value: Uuid) -> Self {
                Self(value)
            }
        }
    };
}

id_newtype!(TenantId);
id_newtype!(CompanyId);
id_newtype!(OrgUnitId);
id_newtype!(UserId);
id_newtype!(PrincipalId);
id_newtype!(SessionId);
id_newtype!(RoleId);
id_newtype!(GrantId);
id_newtype!(TeamId);
id_newtype!(FileObjectId);
id_newtype!(AuditEntryId);
id_newtype!(LicenseId);
id_newtype!(ProjectMembershipId);
id_newtype!(PermissionOverrideId);

id_newtype!(PersonId); // Reference only — authority lives in Workforce / People.
id_newtype!(ProjectId); // Reference only — authority lives in Projects.

id_newtype!(CorrelationId);
id_newtype!(CausationId);

/// Stable permission code owned by Core (`core.user.manage`, `safety.activity.create`, …).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PermissionCode(pub String);

impl PermissionCode {
    pub fn new(code: impl Into<String>) -> Self {
        Self(code.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PermissionCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for PermissionCode {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

/// ISO-like region code (CA, US, AU, NZ, …).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RegionCode(pub String);

impl RegionCode {
    pub fn new(code: impl Into<String>) -> Self {
        Self(code.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FeatureFlagKey(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SettingKey(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ModuleKey(pub String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip_display() {
        let t = TenantId::new();
        assert_eq!(t.to_string(), t.0.to_string());
        let p = PermissionCode::from("core.user.manage");
        assert_eq!(p.as_str(), "core.user.manage");
    }
}
