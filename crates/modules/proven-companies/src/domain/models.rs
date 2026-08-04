//! Companies aggregates, entities, and value objects — mirrors
//! `db/migrations/companies/20260803210000_companies_schema.sql` (ADR-0005).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use proven_shared::{CompanyId, FileObjectId, TenantId, UserId};

use super::enums::{
    AddressKind, BusinessUnitStatus, ContactKind, DigestCadence, MeasurementSystem, ProfileStatus,
    TemplateKind,
};
use super::ids::{AddressId, BusinessUnitId, ContactId, DefaultTemplateId};

/// Profile shell for a Core `Company` — one row per `CompanyId` (`companies.company_profiles`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyProfile {
    pub company_id: CompanyId,
    pub tenant_id: TenantId,
    pub status: ProfileStatus,
    pub trade_name: Option<String>,
    pub website: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

/// Company-scoped hierarchical business unit (`companies.business_units`), distinct from Core's
/// tenant-wide `OrgUnit` tree. `org_unit_id` is an optional, unenforced UUID reference to a Core
/// `OrgUnit` — never dereferenced by this module (ADR-0005: UUID refs only, no cross-schema FK).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessUnit {
    pub id: BusinessUnitId,
    pub company_id: CompanyId,
    pub tenant_id: TenantId,
    pub parent_id: Option<BusinessUnitId>,
    pub org_unit_id: Option<Uuid>,
    pub name: String,
    pub code: Option<String>,
    pub status: BusinessUnitStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

/// A physical/mailing address belonging to a company (`companies.addresses`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub id: AddressId,
    pub company_id: CompanyId,
    pub tenant_id: TenantId,
    pub business_unit_id: Option<BusinessUnitId>,
    pub kind: AddressKind,
    pub line1: String,
    pub line2: Option<String>,
    pub city: String,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    /// ISO 3166-1 alpha-2 country code (validated to exactly 2 characters).
    pub country_code: String,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A named point of contact for a company (`companies.contacts`). `user_id` is an optional,
/// unenforced UUID reference to a Core `User` — never dereferenced by this module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: ContactId,
    pub company_id: CompanyId,
    pub tenant_id: TenantId,
    pub business_unit_id: Option<BusinessUnitId>,
    pub kind: ContactKind,
    pub full_name: String,
    pub title: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub user_id: Option<UserId>,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Visual branding applied to a company's documents, reports, and portals (`companies.branding`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyBranding {
    pub company_id: CompanyId,
    pub tenant_id: TenantId,
    pub logo_file_id: Option<FileObjectId>,
    pub wordmark_file_id: Option<FileObjectId>,
    /// `#RRGGBB` hex color, if set.
    pub primary_color: Option<String>,
    /// `#RRGGBB` hex color, if set.
    pub secondary_color: Option<String>,
    /// `#RRGGBB` hex color, if set.
    pub accent_color: Option<String>,
    pub favicon_file_id: Option<FileObjectId>,
    pub updated_at: DateTime<Utc>,
}

impl CompanyBranding {
    pub fn defaults(company_id: CompanyId, tenant_id: TenantId, now: DateTime<Utc>) -> Self {
        Self {
            company_id,
            tenant_id,
            logo_file_id: None,
            wordmark_file_id: None,
            primary_color: None,
            secondary_color: None,
            accent_color: None,
            favicon_file_id: None,
            updated_at: now,
        }
    }
}

/// Company-wide safety program defaults (`companies.safety_settings`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetySettings {
    pub company_id: CompanyId,
    pub tenant_id: TenantId,
    pub require_flha_before_work: bool,
    pub require_toolbox_talk_weekly: bool,
    pub incident_notify_emails: Vec<String>,
    pub default_risk_matrix: String,
    pub allow_offline_safety_submit: bool,
    pub updated_at: DateTime<Utc>,
}

impl SafetySettings {
    pub fn defaults(company_id: CompanyId, tenant_id: TenantId, now: DateTime<Utc>) -> Self {
        Self {
            company_id,
            tenant_id,
            require_flha_before_work: true,
            require_toolbox_talk_weekly: false,
            incident_notify_emails: Vec::new(),
            default_risk_matrix: "standard".to_string(),
            allow_offline_safety_submit: true,
            updated_at: now,
        }
    }
}

/// Regional/localization defaults for a company (`companies.regional_settings`; distinct from
/// Core `Tenant.region_code`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionalSettings {
    pub company_id: CompanyId,
    pub tenant_id: TenantId,
    pub primary_region: String,
    pub locales: Vec<String>,
    /// IANA timezone name, e.g. `America/Vancouver`.
    pub timezone: String,
    pub measurement_system: MeasurementSystem,
    /// ISO 4217 currency code, e.g. `CAD`.
    pub currency_code: String,
    pub updated_at: DateTime<Utc>,
}

impl RegionalSettings {
    pub fn defaults(company_id: CompanyId, tenant_id: TenantId, now: DateTime<Utc>) -> Self {
        Self {
            company_id,
            tenant_id,
            primary_region: "CA".to_string(),
            locales: vec!["en".to_string()],
            timezone: "UTC".to_string(),
            measurement_system: MeasurementSystem::Metric,
            currency_code: "CAD".to_string(),
            updated_at: now,
        }
    }
}

/// Pointer to a default document template a company uses for a given `TemplateKind`
/// (`companies.default_templates`). The template artifact itself is owned by
/// Documents/Training/Projects/Safety (`domain::ownership`) — `template_ref` is an opaque
/// pointer, never dereferenced by this module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultTemplate {
    pub id: DefaultTemplateId,
    pub company_id: CompanyId,
    pub tenant_id: TenantId,
    pub kind: TemplateKind,
    pub template_ref: String,
    pub label: Option<String>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Company-wide notification defaults (`companies.notification_defaults`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationDefaults {
    pub company_id: CompanyId,
    pub tenant_id: TenantId,
    pub email_enabled: bool,
    pub push_enabled: bool,
    pub sms_enabled: bool,
    pub digest_cadence: DigestCadence,
    /// `HH:MM` (24h), if quiet hours are configured.
    pub quiet_hours_start: Option<String>,
    /// `HH:MM` (24h), if quiet hours are configured.
    pub quiet_hours_end: Option<String>,
    pub default_locale: String,
    pub updated_at: DateTime<Utc>,
}

impl NotificationDefaults {
    pub fn defaults(company_id: CompanyId, tenant_id: TenantId, now: DateTime<Utc>) -> Self {
        Self {
            company_id,
            tenant_id,
            email_enabled: true,
            push_enabled: true,
            sms_enabled: false,
            digest_cadence: DigestCadence::Daily,
            quiet_hours_start: None,
            quiet_hours_end: None,
            default_locale: "en".to_string(),
            updated_at: now,
        }
    }
}

/// Company-wide file upload/storage policy (`companies.storage_configuration`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfiguration {
    pub company_id: CompanyId,
    pub tenant_id: TenantId,
    pub object_prefix: String,
    pub max_upload_bytes: i64,
    pub allowed_content_types: Vec<String>,
    pub retention_class_default: String,
    pub quarantine_enabled: bool,
    pub updated_at: DateTime<Utc>,
}

impl StorageConfiguration {
    pub fn defaults(company_id: CompanyId, tenant_id: TenantId, now: DateTime<Utc>) -> Self {
        Self {
            company_id,
            tenant_id,
            object_prefix: format!("companies/{}/", company_id.as_uuid()),
            max_upload_bytes: 52_428_800,
            allowed_content_types: vec![
                "application/pdf".to_string(),
                "image/jpeg".to_string(),
                "image/png".to_string(),
            ],
            retention_class_default: "standard".to_string(),
            quarantine_enabled: true,
            updated_at: now,
        }
    }
}
