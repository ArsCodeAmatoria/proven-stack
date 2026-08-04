//! Stable Users permission codes — matches the SQL seed
//! `db/migrations/users/20260803220001_users_permissions.sql` (ADR-0006).
//!
//! Permission codes are append-only. Never rename in place. These codes are published into
//! Core's permission catalog but Core never interprets them — AuthZ decisions still flow
//! exclusively through `proven_core::AuthzApi` (ADR-0003).

pub const PROFILE_READ: &str = "users.profile.read";
pub const PROFILE_MANAGE: &str = "users.profile.manage";
pub const KIND_MANAGE: &str = "users.kind.manage";
pub const AVATAR_MANAGE: &str = "users.avatar.manage";
pub const PREFERENCES_MANAGE: &str = "users.preferences.manage";
pub const AUTH_PROFILE_MANAGE: &str = "users.auth_profile.manage";
pub const SIGNATURE_PROFILE_MANAGE: &str = "users.signature_profile.manage";
pub const EMERGENCY_CONTACT_MANAGE: &str = "users.emergency_contact.manage";
pub const SETTINGS_MANAGE: &str = "users.settings.manage";
pub const AUDIT_READ: &str = "users.audit.read";

/// Full catalog of Users-owned (`users.*`) permission codes.
pub const ALL_USERS_PERMISSIONS: &[&str] = &[
    PROFILE_READ,
    PROFILE_MANAGE,
    KIND_MANAGE,
    AVATAR_MANAGE,
    PREFERENCES_MANAGE,
    AUTH_PROFILE_MANAGE,
    SIGNATURE_PROFILE_MANAGE,
    EMERGENCY_CONTACT_MANAGE,
    SETTINGS_MANAGE,
    AUDIT_READ,
];
