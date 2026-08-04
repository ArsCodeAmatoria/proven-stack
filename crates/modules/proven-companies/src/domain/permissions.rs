//! Stable Companies permission codes — matches the (future) SQL seed
//! `db/migrations/companies/*_companies_permissions_seed.sql` (ADR-0005 §5).
//!
//! Permission codes are append-only. Never rename in place. These codes are published into
//! Core's permission catalog but Core never interprets them — AuthZ decisions still flow
//! exclusively through `proven_core::AuthzApi` (ADR-0003).

pub const PROFILE_READ: &str = "companies.profile.read";
pub const PROFILE_MANAGE: &str = "companies.profile.manage";
pub const UNIT_MANAGE: &str = "companies.unit.manage";
pub const ADDRESS_MANAGE: &str = "companies.address.manage";
pub const CONTACT_MANAGE: &str = "companies.contact.manage";
pub const BRANDING_MANAGE: &str = "companies.branding.manage";
pub const SAFETY_SETTINGS_MANAGE: &str = "companies.safety_settings.manage";
pub const REGIONAL_SETTINGS_MANAGE: &str = "companies.regional_settings.manage";
pub const TEMPLATES_MANAGE: &str = "companies.templates.manage";
pub const NOTIFICATION_DEFAULTS_MANAGE: &str = "companies.notification_defaults.manage";
pub const STORAGE_MANAGE: &str = "companies.storage.manage";

/// Full catalog of Companies-owned (`companies.*`) permission codes.
pub const ALL_COMPANIES_PERMISSIONS: &[&str] = &[
    PROFILE_READ,
    PROFILE_MANAGE,
    UNIT_MANAGE,
    ADDRESS_MANAGE,
    CONTACT_MANAGE,
    BRANDING_MANAGE,
    SAFETY_SETTINGS_MANAGE,
    REGIONAL_SETTINGS_MANAGE,
    TEMPLATES_MANAGE,
    NOTIFICATION_DEFAULTS_MANAGE,
    STORAGE_MANAGE,
];
