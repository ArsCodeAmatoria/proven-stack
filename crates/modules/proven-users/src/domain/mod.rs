//! Users domain layer — pure types and rules, no I/O (ADR-0006).

mod enums;
mod error;
mod ids;
mod models;
pub mod ownership;
pub mod permissions;
pub mod validation;

pub use enums::{DigestCadence, ProfileStatus, SignatureType, UserKind};
pub use error::UsersError;
pub use ids::{EmergencyContactId, ProfileAuditEntryId, UserKindAssignmentId};
pub use models::{
    AccessibilityPreferences, AuthenticationProfile, Avatar, DigitalSignatureProfile,
    EmergencyContact, LocalePreferences, NotificationPreferences, ProfileAuditEntry,
    UserKindAssignment, UserProfile, UserSetting,
};
