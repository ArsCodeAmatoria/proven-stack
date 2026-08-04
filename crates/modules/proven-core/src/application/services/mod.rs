//! Application service implementations. Each service depends only on `application::ports`
//! traits, so it works identically against `infrastructure::memory` or `infrastructure::postgres`.

pub mod audit_service;
pub mod authz_service;
pub mod file_service;
pub mod flags_service;
pub mod identity_service;
pub mod license_service;
pub mod membership_service;
pub mod settings_service;
pub mod tenancy_service;

pub use audit_service::{AppendAuditEntryCommand, AuditEngine, AuditService};
pub use authz_service::{
    AuthorizeRequest, AuthzService, GrantAccessCommand, UpsertPermissionOverrideCommand,
};
pub use file_service::{
    ApplyScanResultCommand, CreateFileUploadIntentCommand, CreatePublicShareLinkCommand,
    FileService,
};
pub use flags_service::FlagsService;
pub use identity_service::{IdentityService, InviteUserCommand};
pub use license_service::LicenseService;
pub use membership_service::{CreateTeamCommand, GrantProjectMembershipCommand, MembershipService};
pub use settings_service::{SettingsService, UpsertSettingCommand};
pub use tenancy_service::{
    ProvisionTenantCommand, ProvisionTenantResult, RegisterCompanyCommand, TenancyService,
};
