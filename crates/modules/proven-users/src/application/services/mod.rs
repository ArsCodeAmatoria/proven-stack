//! Application service implementations. Each service depends only on `application::ports`
//! traits, so it works identically against `infrastructure::memory` or a future
//! `infrastructure::postgres` adapter (ADR-0006, mirrors ADR-0004 for Core / ADR-0005 for
//! Companies).

pub mod accessibility_service;
pub mod audit_history_service;
pub mod audit_recorder;
pub mod auth_profile_service;
pub mod authz;
pub mod avatar_service;
pub mod emergency_contact_service;
pub mod kind_service;
pub mod locale_service;
pub mod notification_service;
pub mod profile_service;
pub mod settings_service;
pub mod signature_service;

pub use accessibility_service::{AccessibilityService, UpsertAccessibilityCommand};
pub use audit_history_service::AuditHistoryService;
pub use audit_recorder::AuditRecorder;
pub use auth_profile_service::{AuthProfileService, UpsertAuthenticationProfileCommand};
pub use authz::{authorize, authorize_self_or_permission, ActingContext, AllowAllAuthz};
pub use avatar_service::{AvatarService, UpsertAvatarCommand};
pub use emergency_contact_service::{
    AddEmergencyContactCommand, EmergencyContactService, UpdateEmergencyContactCommand,
};
pub use kind_service::{AssignUserKindCommand, KindService};
pub use locale_service::{LocaleService, UpsertLocaleCommand};
pub use notification_service::{NotificationService, UpsertNotificationPreferencesCommand};
pub use profile_service::{ProfileService, UpdateProfileCommand};
pub use settings_service::{SettingsService, UpsertUserSettingCommand};
pub use signature_service::{SignatureService, UpsertSignatureProfileCommand};
