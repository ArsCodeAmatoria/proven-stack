//! Application service implementations. Each service depends only on `application::ports`
//! traits, so it works identically against `infrastructure::memory` or a future
//! `infrastructure::postgres` adapter (ADR-0005, mirrors ADR-0004 for Core).

pub mod address_service;
pub mod authz;
pub mod branding_service;
pub mod business_unit_service;
pub mod contact_service;
pub mod notification_defaults_service;
pub mod profile_service;
pub mod regional_settings_service;
pub mod safety_settings_service;
pub mod storage_service;
pub mod templates_service;

pub use address_service::{AddAddressCommand, AddressService, UpdateAddressCommand};
pub use authz::{ActingContext, AllowAllAuthz};
pub use branding_service::{BrandingService, UpsertBrandingCommand};
pub use business_unit_service::{
    BusinessUnitService, CreateBusinessUnitCommand, UpdateBusinessUnitCommand,
};
pub use contact_service::{AddContactCommand, ContactService, UpdateContactCommand};
pub use notification_defaults_service::{
    NotificationDefaultsService, UpsertNotificationDefaultsCommand,
};
pub use profile_service::{ProfileService, UpdateProfileCommand};
pub use regional_settings_service::{RegionalSettingsService, UpsertRegionalSettingsCommand};
pub use safety_settings_service::{SafetySettingsService, UpsertSafetySettingsCommand};
pub use storage_service::{StorageService, UpsertStorageConfigurationCommand};
pub use templates_service::{TemplatesService, UpsertDefaultTemplateCommand};
