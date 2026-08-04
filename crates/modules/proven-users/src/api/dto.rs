//! HTTP request/response DTOs. Domain models already derive `Serialize`/`Deserialize` and are
//! returned directly; this module holds request bodies only.

use serde::Deserialize;
use uuid::Uuid;

use crate::domain::{DigestCadence, SignatureType};

#[derive(Debug, Deserialize)]
pub struct EnsureProfileRequest {
    pub display_name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub preferred_name: Option<String>,
    #[serde(default)]
    pub job_title: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub bio: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignKindRequest {
    pub kind: crate::domain::UserKind,
    #[serde(default)]
    pub is_primary: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpsertAvatarRequest {
    #[serde(default)]
    pub file_object_id: Option<Uuid>,
    #[serde(default)]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertLocaleRequest {
    #[serde(default)]
    pub language_code: Option<String>,
    #[serde(default)]
    pub time_zone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertAccessibilityRequest {
    #[serde(default)]
    pub reduce_motion: Option<bool>,
    #[serde(default)]
    pub high_contrast: Option<bool>,
    #[serde(default)]
    pub large_text: Option<bool>,
    #[serde(default)]
    pub screen_reader_hints: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertNotificationPreferencesRequest {
    #[serde(default)]
    pub email_enabled: Option<bool>,
    #[serde(default)]
    pub push_enabled: Option<bool>,
    #[serde(default)]
    pub sms_enabled: Option<bool>,
    #[serde(default)]
    pub in_app_enabled: Option<bool>,
    #[serde(default)]
    pub digest_cadence: Option<DigestCadence>,
    #[serde(default)]
    pub quiet_hours_start: Option<String>,
    #[serde(default)]
    pub quiet_hours_end: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertAuthenticationProfileRequest {
    #[serde(default)]
    pub mfa_preferred: Option<bool>,
    #[serde(default)]
    pub password_login_enabled: Option<bool>,
    #[serde(default)]
    pub oauth_google_linked: Option<bool>,
    #[serde(default)]
    pub oauth_microsoft_linked: Option<bool>,
    #[serde(default)]
    pub magic_link_preferred: Option<bool>,
    #[serde(default)]
    pub last_auth_method: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertSignatureProfileRequest {
    #[serde(default)]
    pub default_signature_type: Option<SignatureType>,
    #[serde(default)]
    pub typed_name_default: Option<String>,
    #[serde(default)]
    pub signature_image_file_id: Option<Uuid>,
    #[serde(default)]
    pub require_reauth_to_sign: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AddEmergencyContactRequest {
    pub full_name: String,
    #[serde(default)]
    pub relationship: Option<String>,
    pub phone: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub is_primary: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEmergencyContactRequest {
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub relationship: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub is_primary: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertSettingRequest {
    pub value: serde_json::Value,
}
