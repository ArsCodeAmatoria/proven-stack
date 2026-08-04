//! HTTP request/response DTOs. Domain models already derive `Serialize`/`Deserialize` and are
//! returned directly; this module holds request bodies only.

use serde::Deserialize;
use uuid::Uuid;

use crate::domain::{AddressKind, ContactKind, DigestCadence, MeasurementSystem, TemplateKind};

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    #[serde(default)]
    pub trade_name: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBusinessUnitRequest {
    pub name: String,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub parent_id: Option<Uuid>,
    #[serde(default)]
    pub org_unit_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBusinessUnitRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub parent_id: Option<Uuid>,
    #[serde(default)]
    pub org_unit_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct AddAddressRequest {
    #[serde(default)]
    pub business_unit_id: Option<Uuid>,
    pub kind: AddressKind,
    pub line1: String,
    #[serde(default)]
    pub line2: Option<String>,
    pub city: String,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub postal_code: Option<String>,
    pub country_code: String,
    #[serde(default)]
    pub is_primary: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAddressRequest {
    #[serde(default)]
    pub business_unit_id: Option<Uuid>,
    #[serde(default)]
    pub kind: Option<AddressKind>,
    #[serde(default)]
    pub line1: Option<String>,
    #[serde(default)]
    pub line2: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub postal_code: Option<String>,
    #[serde(default)]
    pub country_code: Option<String>,
    #[serde(default)]
    pub is_primary: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AddContactRequest {
    #[serde(default)]
    pub business_unit_id: Option<Uuid>,
    pub kind: ContactKind,
    pub full_name: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub user_id: Option<Uuid>,
    #[serde(default)]
    pub is_primary: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateContactRequest {
    #[serde(default)]
    pub business_unit_id: Option<Uuid>,
    #[serde(default)]
    pub kind: Option<ContactKind>,
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub user_id: Option<Uuid>,
    #[serde(default)]
    pub is_primary: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertBrandingRequest {
    #[serde(default)]
    pub logo_file_id: Option<Uuid>,
    #[serde(default)]
    pub wordmark_file_id: Option<Uuid>,
    #[serde(default)]
    pub primary_color: Option<String>,
    #[serde(default)]
    pub secondary_color: Option<String>,
    #[serde(default)]
    pub accent_color: Option<String>,
    #[serde(default)]
    pub favicon_file_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertSafetySettingsRequest {
    #[serde(default)]
    pub require_flha_before_work: Option<bool>,
    #[serde(default)]
    pub require_toolbox_talk_weekly: Option<bool>,
    #[serde(default)]
    pub incident_notify_emails: Option<Vec<String>>,
    #[serde(default)]
    pub default_risk_matrix: Option<String>,
    #[serde(default)]
    pub allow_offline_safety_submit: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertRegionalSettingsRequest {
    #[serde(default)]
    pub primary_region: Option<String>,
    #[serde(default)]
    pub locales: Option<Vec<String>>,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub measurement_system: Option<MeasurementSystem>,
    #[serde(default)]
    pub currency_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertDefaultTemplateRequest {
    pub kind: TemplateKind,
    pub template_ref: String,
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertNotificationDefaultsRequest {
    #[serde(default)]
    pub email_enabled: Option<bool>,
    #[serde(default)]
    pub push_enabled: Option<bool>,
    #[serde(default)]
    pub sms_enabled: Option<bool>,
    #[serde(default)]
    pub digest_cadence: Option<DigestCadence>,
    #[serde(default)]
    pub quiet_hours_start: Option<String>,
    #[serde(default)]
    pub quiet_hours_end: Option<String>,
    #[serde(default)]
    pub default_locale: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertStorageConfigurationRequest {
    #[serde(default)]
    pub object_prefix: Option<String>,
    #[serde(default)]
    pub max_upload_bytes: Option<i64>,
    #[serde(default)]
    pub allowed_content_types: Option<Vec<String>>,
    #[serde(default)]
    pub retention_class_default: Option<String>,
    #[serde(default)]
    pub quarantine_enabled: Option<bool>,
}
