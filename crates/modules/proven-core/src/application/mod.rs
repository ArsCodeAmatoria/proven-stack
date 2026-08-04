//! Application layer — public interfaces (CORE_DOMAIN.md §13.1) and their implementations.
//! No HTTP, no SQL: those live in `api` and `infrastructure` respectively.

pub mod apis;
pub mod ports;
pub mod services;

pub use apis::{
    AuditApi, AuthzApi, CorePorts, CoreServices, FileApi, FlagsApi, IdentityApi, LicenseApi,
    MembershipApi, SettingsApi, TenancyApi,
};
