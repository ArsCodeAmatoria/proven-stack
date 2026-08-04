//! Users aggregates, entities, and value objects — mirrors
//! `db/migrations/users/20260803220000_users_schema.sql` (ADR-0006).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use proven_shared::{CompanyId, FileObjectId, PersonId, TenantId, UserId};

use super::enums::{DigestCadence, ProfileStatus, SignatureType, UserKind};
use super::ids::{EmergencyContactId, ProfileAuditEntryId, UserKindAssignmentId};

/// Account profile shell for a Core `User` — one row per `UserId` (`users.user_profiles`).
/// Core remains the System of Record for login identity; this module owns everything below.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub status: ProfileStatus,
    pub display_name: String,
    pub preferred_name: Option<String>,
    pub job_title: Option<String>,
    pub phone: Option<String>,
    /// Optional, unenforced UUID reference to a `proven-companies` `CompanyId` — never
    /// dereferenced by this module (ADR-0006: UUID refs only, no cross-schema FK).
    pub company_id: Option<CompanyId>,
    /// Optional, unenforced reference to a future People workforce `PersonId` — never
    /// dereferenced by this module. Linking `PersonId` ↔ `UserId` is People's job, not Users'.
    pub person_id: Option<PersonId>,
    pub bio: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

/// A single `UserKind` classification tag assigned to a user (`users.user_kinds`). At most one
/// assignment per user may have `is_primary = true` (enforced by the application layer, not the
/// in-memory store — see `application::services::kind_service`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserKindAssignment {
    pub id: UserKindAssignmentId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub kind: UserKind,
    pub is_primary: bool,
    pub assigned_at: DateTime<Utc>,
}

/// A user's avatar image pointer (`users.avatars`). Absent until explicitly set — not part of
/// `ensure_profile`'s default provisioning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Avatar {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub file_object_id: Option<FileObjectId>,
    pub avatar_url: Option<String>,
    pub updated_at: DateTime<Utc>,
}

/// Language/timezone preferences (`users.locale_preferences`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalePreferences {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub language_code: String,
    pub time_zone: String,
    pub updated_at: DateTime<Utc>,
}

impl LocalePreferences {
    pub fn defaults(user_id: UserId, tenant_id: TenantId, now: DateTime<Utc>) -> Self {
        Self {
            user_id,
            tenant_id,
            language_code: "en".to_string(),
            time_zone: "UTC".to_string(),
            updated_at: now,
        }
    }
}

/// Accessibility preferences (`users.accessibility_preferences`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityPreferences {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub reduce_motion: bool,
    pub high_contrast: bool,
    pub large_text: bool,
    pub screen_reader_hints: bool,
    pub updated_at: DateTime<Utc>,
}

impl AccessibilityPreferences {
    pub fn defaults(user_id: UserId, tenant_id: TenantId, now: DateTime<Utc>) -> Self {
        Self {
            user_id,
            tenant_id,
            reduce_motion: false,
            high_contrast: false,
            large_text: false,
            screen_reader_hints: false,
            updated_at: now,
        }
    }
}

/// Notification channel preferences (`users.notification_preferences`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub email_enabled: bool,
    pub push_enabled: bool,
    pub sms_enabled: bool,
    pub in_app_enabled: bool,
    pub digest_cadence: DigestCadence,
    /// `HH:MM` (24h), if quiet hours are configured.
    pub quiet_hours_start: Option<String>,
    /// `HH:MM` (24h), if quiet hours are configured.
    pub quiet_hours_end: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl NotificationPreferences {
    pub fn defaults(user_id: UserId, tenant_id: TenantId, now: DateTime<Utc>) -> Self {
        Self {
            user_id,
            tenant_id,
            email_enabled: true,
            push_enabled: true,
            sms_enabled: false,
            in_app_enabled: true,
            digest_cadence: DigestCadence::Daily,
            quiet_hours_start: None,
            quiet_hours_end: None,
            updated_at: now,
        }
    }
}

/// Authentication *preference* mirror flags (`users.authentication_profiles`). **Never** stores
/// password hashes, credentials, or session material — Core (`proven_core`) remains the sole
/// System of Record for authentication secrets (ADR-0006 §6, `domain::ownership`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationProfile {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub mfa_preferred: bool,
    pub password_login_enabled: bool,
    pub oauth_google_linked: bool,
    pub oauth_microsoft_linked: bool,
    pub magic_link_preferred: bool,
    pub last_auth_method: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl AuthenticationProfile {
    pub fn defaults(user_id: UserId, tenant_id: TenantId, now: DateTime<Utc>) -> Self {
        Self {
            user_id,
            tenant_id,
            mfa_preferred: false,
            password_login_enabled: true,
            oauth_google_linked: false,
            oauth_microsoft_linked: false,
            magic_link_preferred: false,
            last_auth_method: None,
            updated_at: now,
        }
    }
}

/// Digital signing preferences/assurance hints (`users.digital_signature_profiles`) — **not**
/// signature packages or guest signing tokens, which stay in the (future) Signatures module
/// (ADR-0006 §5/§7, `domain::ownership`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalSignatureProfile {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub default_signature_type: SignatureType,
    pub typed_name_default: Option<String>,
    pub signature_image_file_id: Option<FileObjectId>,
    pub require_reauth_to_sign: bool,
    pub updated_at: DateTime<Utc>,
}

impl DigitalSignatureProfile {
    pub fn defaults(user_id: UserId, tenant_id: TenantId, now: DateTime<Utc>) -> Self {
        Self {
            user_id,
            tenant_id,
            default_signature_type: SignatureType::Drawn,
            typed_name_default: None,
            signature_image_file_id: None,
            require_reauth_to_sign: false,
            updated_at: now,
        }
    }
}

/// An emergency contact for a user (`users.emergency_contacts`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyContact {
    pub id: EmergencyContactId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub full_name: String,
    pub relationship: Option<String>,
    pub phone: String,
    pub email: Option<String>,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A single arbitrary user-scoped key/value setting (`users.user_settings`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSetting {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub key: String,
    pub value: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

/// Append-only profile change history entry (`users.profile_audit_entries`) — a lightweight
/// change log for *this module's* profile data, not a substitute for Core's `AuditApi` (ADR-0006
/// §8).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileAuditEntry {
    pub id: ProfileAuditEntryId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub actor_user_id: Option<UserId>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub summary: String,
    pub payload: serde_json::Value,
    pub occurred_at: DateTime<Utc>,
}
