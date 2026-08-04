//! In-process public interface (ADR-0005 §4). Every other module and Temporal activity talks to
//! Companies exclusively through [`CompaniesApi`] — never through this module's schema.

use std::sync::Arc;

use async_trait::async_trait;
use proven_core::{AuthzApi, TenancyApi};
use proven_shared::CompanyId;

use crate::application::ports::{
    AddressRepository, BrandingRepository, BusinessUnitRepository, CompanyProfileRepository,
    ContactRepository, DefaultTemplateRepository, EventPublisher, NotificationDefaultsRepository,
    RegionalSettingsRepository, SafetySettingsRepository, StorageConfigurationRepository,
};
use crate::application::services::{
    AddAddressCommand, AddContactCommand, ActingContext, AddressService, AllowAllAuthz,
    BrandingService, BusinessUnitService, ContactService, CreateBusinessUnitCommand,
    NotificationDefaultsService, ProfileService, RegionalSettingsService, SafetySettingsService,
    StorageService, TemplatesService, UpdateAddressCommand, UpdateBusinessUnitCommand,
    UpdateContactCommand, UpdateProfileCommand, UpsertBrandingCommand,
    UpsertDefaultTemplateCommand, UpsertNotificationDefaultsCommand,
    UpsertRegionalSettingsCommand, UpsertSafetySettingsCommand, UpsertStorageConfigurationCommand,
};
use crate::domain::{
    Address, AddressId, BusinessUnit, BusinessUnitId, CompaniesError, CompanyBranding,
    CompanyProfile, Contact, ContactId, DefaultTemplate, NotificationDefaults, RegionalSettings,
    SafetySettings, StorageConfiguration,
};
use crate::infrastructure::memory::MemoryStore;
use crate::infrastructure::outbox::InMemoryOutbox;

/// Facade covering every Companies capability (ADR-0005 §4). Mutations take an [`ActingContext`]
/// so implementations can enforce tenant scoping + AuthZ; reads are keyed by `CompanyId` alone.
#[async_trait]
pub trait CompaniesApi: Send + Sync {
    async fn ensure_profile(
        &self,
        ctx: ActingContext,
        company_id: CompanyId,
    ) -> Result<CompanyProfile, CompaniesError>;
    async fn get_profile(&self, company_id: CompanyId) -> Result<CompanyProfile, CompaniesError>;
    async fn update_profile(
        &self,
        ctx: ActingContext,
        cmd: UpdateProfileCommand,
    ) -> Result<CompanyProfile, CompaniesError>;
    async fn archive_profile(
        &self,
        ctx: ActingContext,
        company_id: CompanyId,
    ) -> Result<CompanyProfile, CompaniesError>;

    async fn create_business_unit(
        &self,
        ctx: ActingContext,
        cmd: CreateBusinessUnitCommand,
    ) -> Result<BusinessUnit, CompaniesError>;
    async fn list_business_units(
        &self,
        company_id: CompanyId,
    ) -> Result<Vec<BusinessUnit>, CompaniesError>;
    async fn update_business_unit(
        &self,
        ctx: ActingContext,
        cmd: UpdateBusinessUnitCommand,
    ) -> Result<BusinessUnit, CompaniesError>;
    async fn archive_business_unit(
        &self,
        ctx: ActingContext,
        company_id: CompanyId,
        business_unit_id: BusinessUnitId,
    ) -> Result<BusinessUnit, CompaniesError>;

    async fn add_address(
        &self,
        ctx: ActingContext,
        cmd: AddAddressCommand,
    ) -> Result<Address, CompaniesError>;
    async fn list_addresses(&self, company_id: CompanyId) -> Result<Vec<Address>, CompaniesError>;
    async fn update_address(
        &self,
        ctx: ActingContext,
        cmd: UpdateAddressCommand,
    ) -> Result<Address, CompaniesError>;
    async fn remove_address(
        &self,
        ctx: ActingContext,
        company_id: CompanyId,
        address_id: AddressId,
    ) -> Result<(), CompaniesError>;

    async fn add_contact(
        &self,
        ctx: ActingContext,
        cmd: AddContactCommand,
    ) -> Result<Contact, CompaniesError>;
    async fn list_contacts(&self, company_id: CompanyId) -> Result<Vec<Contact>, CompaniesError>;
    async fn update_contact(
        &self,
        ctx: ActingContext,
        cmd: UpdateContactCommand,
    ) -> Result<Contact, CompaniesError>;
    async fn remove_contact(
        &self,
        ctx: ActingContext,
        company_id: CompanyId,
        contact_id: ContactId,
    ) -> Result<(), CompaniesError>;

    async fn get_branding(&self, company_id: CompanyId) -> Result<CompanyBranding, CompaniesError>;
    async fn upsert_branding(
        &self,
        ctx: ActingContext,
        cmd: UpsertBrandingCommand,
    ) -> Result<CompanyBranding, CompaniesError>;

    async fn get_safety_settings(
        &self,
        company_id: CompanyId,
    ) -> Result<SafetySettings, CompaniesError>;
    async fn upsert_safety_settings(
        &self,
        ctx: ActingContext,
        cmd: UpsertSafetySettingsCommand,
    ) -> Result<SafetySettings, CompaniesError>;

    async fn get_regional_settings(
        &self,
        company_id: CompanyId,
    ) -> Result<RegionalSettings, CompaniesError>;
    async fn upsert_regional_settings(
        &self,
        ctx: ActingContext,
        cmd: UpsertRegionalSettingsCommand,
    ) -> Result<RegionalSettings, CompaniesError>;

    async fn list_default_templates(
        &self,
        company_id: CompanyId,
    ) -> Result<Vec<DefaultTemplate>, CompaniesError>;
    async fn upsert_default_template(
        &self,
        ctx: ActingContext,
        cmd: UpsertDefaultTemplateCommand,
    ) -> Result<DefaultTemplate, CompaniesError>;

    async fn get_notification_defaults(
        &self,
        company_id: CompanyId,
    ) -> Result<NotificationDefaults, CompaniesError>;
    async fn upsert_notification_defaults(
        &self,
        ctx: ActingContext,
        cmd: UpsertNotificationDefaultsCommand,
    ) -> Result<NotificationDefaults, CompaniesError>;

    async fn get_storage_configuration(
        &self,
        company_id: CompanyId,
    ) -> Result<StorageConfiguration, CompaniesError>;
    async fn upsert_storage_configuration(
        &self,
        ctx: ActingContext,
        cmd: UpsertStorageConfigurationCommand,
    ) -> Result<StorageConfiguration, CompaniesError>;
}

/// Bundle of repository ports used to construct [`CompaniesServices`]. Swap
/// `infrastructure::memory` for a future `infrastructure::postgres` adapter without touching
/// application logic (ADR-0005, mirrors ADR-0004 for Core).
pub struct CompaniesPorts {
    pub profiles: Arc<dyn CompanyProfileRepository>,
    pub business_units: Arc<dyn BusinessUnitRepository>,
    pub addresses: Arc<dyn AddressRepository>,
    pub contacts: Arc<dyn ContactRepository>,
    pub branding: Arc<dyn BrandingRepository>,
    pub safety_settings: Arc<dyn SafetySettingsRepository>,
    pub regional_settings: Arc<dyn RegionalSettingsRepository>,
    pub default_templates: Arc<dyn DefaultTemplateRepository>,
    pub notification_defaults: Arc<dyn NotificationDefaultsRepository>,
    pub storage_configuration: Arc<dyn StorageConfigurationRepository>,
    pub outbox: Arc<dyn EventPublisher>,
}

impl CompaniesPorts {
    /// Wire every port to a single shared in-memory store (unit tests / no-DB mode).
    pub fn in_memory() -> Self {
        let store = Arc::new(MemoryStore::new());
        let outbox = Arc::new(InMemoryOutbox::new());
        Self {
            profiles: store.clone(),
            business_units: store.clone(),
            addresses: store.clone(),
            contacts: store.clone(),
            branding: store.clone(),
            safety_settings: store.clone(),
            regional_settings: store.clone(),
            default_templates: store.clone(),
            notification_defaults: store.clone(),
            storage_configuration: store,
            outbox,
        }
    }
}

/// Facade implementing [`CompaniesApi`] — the seam other modules depend on.
pub struct CompaniesServices {
    profile: ProfileService,
    business_units: BusinessUnitService,
    addresses: AddressService,
    contacts: ContactService,
    branding: BrandingService,
    safety_settings: SafetySettingsService,
    regional_settings: RegionalSettingsService,
    templates: TemplatesService,
    notification_defaults: NotificationDefaultsService,
    storage: StorageService,
}

impl CompaniesServices {
    /// Build `CompaniesServices` from ports plus an AuthZ decision path, optionally wiring a
    /// `TenancyApi` so `ensure_profile` can verify the Core company exists first.
    pub fn new(
        ports: CompaniesPorts,
        authz: Arc<dyn AuthzApi>,
        tenancy: Option<Arc<dyn TenancyApi>>,
    ) -> Self {
        Self {
            profile: ProfileService::new(
                ports.profiles,
                ports.safety_settings.clone(),
                ports.regional_settings.clone(),
                ports.notification_defaults.clone(),
                ports.storage_configuration.clone(),
                ports.outbox.clone(),
                authz.clone(),
                tenancy,
            ),
            business_units: BusinessUnitService::new(
                ports.business_units,
                ports.outbox.clone(),
                authz.clone(),
            ),
            addresses: AddressService::new(ports.addresses, ports.outbox.clone(), authz.clone()),
            contacts: ContactService::new(ports.contacts, ports.outbox.clone(), authz.clone()),
            branding: BrandingService::new(ports.branding, ports.outbox.clone(), authz.clone()),
            safety_settings: SafetySettingsService::new(
                ports.safety_settings,
                ports.outbox.clone(),
                authz.clone(),
            ),
            regional_settings: RegionalSettingsService::new(
                ports.regional_settings,
                ports.outbox.clone(),
                authz.clone(),
            ),
            templates: TemplatesService::new(
                ports.default_templates,
                ports.outbox.clone(),
                authz.clone(),
            ),
            notification_defaults: NotificationDefaultsService::new(
                ports.notification_defaults,
                ports.outbox.clone(),
                authz.clone(),
            ),
            storage: StorageService::new(ports.storage_configuration, ports.outbox, authz),
        }
    }

    /// In-memory ports + a stub Allow-all AuthZ, no `TenancyApi` wired (so `ensure_profile`
    /// skips the Core company existence check). Intended for this crate's own unit tests.
    pub fn in_memory_unchecked() -> Self {
        Self::new(CompaniesPorts::in_memory(), Arc::new(AllowAllAuthz), None)
    }

    /// Wire real `proven-core` `AuthzApi` + `TenancyApi` (`CoreServices` implements both) over
    /// in-memory ports. Use this path outside of unit tests.
    pub fn with_core<C>(ports: CompaniesPorts, core: Arc<C>) -> Self
    where
        C: AuthzApi + TenancyApi + Send + Sync + 'static,
    {
        let authz: Arc<dyn AuthzApi> = core.clone();
        let tenancy: Arc<dyn TenancyApi> = core;
        Self::new(ports, authz, Some(tenancy))
    }
}

#[async_trait]
impl CompaniesApi for CompaniesServices {
    async fn ensure_profile(
        &self,
        ctx: ActingContext,
        company_id: CompanyId,
    ) -> Result<CompanyProfile, CompaniesError> {
        self.profile.ensure_profile(&ctx, company_id).await
    }

    async fn get_profile(&self, company_id: CompanyId) -> Result<CompanyProfile, CompaniesError> {
        self.profile.get_profile(company_id).await
    }

    async fn update_profile(
        &self,
        ctx: ActingContext,
        cmd: UpdateProfileCommand,
    ) -> Result<CompanyProfile, CompaniesError> {
        self.profile.update_profile(&ctx, cmd).await
    }

    async fn archive_profile(
        &self,
        ctx: ActingContext,
        company_id: CompanyId,
    ) -> Result<CompanyProfile, CompaniesError> {
        self.profile.archive_profile(&ctx, company_id).await
    }

    async fn create_business_unit(
        &self,
        ctx: ActingContext,
        cmd: CreateBusinessUnitCommand,
    ) -> Result<BusinessUnit, CompaniesError> {
        self.business_units.create(&ctx, cmd).await
    }

    async fn list_business_units(
        &self,
        company_id: CompanyId,
    ) -> Result<Vec<BusinessUnit>, CompaniesError> {
        self.business_units.list(company_id).await
    }

    async fn update_business_unit(
        &self,
        ctx: ActingContext,
        cmd: UpdateBusinessUnitCommand,
    ) -> Result<BusinessUnit, CompaniesError> {
        self.business_units.update(&ctx, cmd).await
    }

    async fn archive_business_unit(
        &self,
        ctx: ActingContext,
        company_id: CompanyId,
        business_unit_id: BusinessUnitId,
    ) -> Result<BusinessUnit, CompaniesError> {
        self.business_units
            .archive(&ctx, company_id, business_unit_id)
            .await
    }

    async fn add_address(
        &self,
        ctx: ActingContext,
        cmd: AddAddressCommand,
    ) -> Result<Address, CompaniesError> {
        self.addresses.add(&ctx, cmd).await
    }

    async fn list_addresses(&self, company_id: CompanyId) -> Result<Vec<Address>, CompaniesError> {
        self.addresses.list(company_id).await
    }

    async fn update_address(
        &self,
        ctx: ActingContext,
        cmd: UpdateAddressCommand,
    ) -> Result<Address, CompaniesError> {
        self.addresses.update(&ctx, cmd).await
    }

    async fn remove_address(
        &self,
        ctx: ActingContext,
        company_id: CompanyId,
        address_id: AddressId,
    ) -> Result<(), CompaniesError> {
        self.addresses.remove(&ctx, company_id, address_id).await
    }

    async fn add_contact(
        &self,
        ctx: ActingContext,
        cmd: AddContactCommand,
    ) -> Result<Contact, CompaniesError> {
        self.contacts.add(&ctx, cmd).await
    }

    async fn list_contacts(&self, company_id: CompanyId) -> Result<Vec<Contact>, CompaniesError> {
        self.contacts.list(company_id).await
    }

    async fn update_contact(
        &self,
        ctx: ActingContext,
        cmd: UpdateContactCommand,
    ) -> Result<Contact, CompaniesError> {
        self.contacts.update(&ctx, cmd).await
    }

    async fn remove_contact(
        &self,
        ctx: ActingContext,
        company_id: CompanyId,
        contact_id: ContactId,
    ) -> Result<(), CompaniesError> {
        self.contacts.remove(&ctx, company_id, contact_id).await
    }

    async fn get_branding(&self, company_id: CompanyId) -> Result<CompanyBranding, CompaniesError> {
        self.branding.get(company_id).await
    }

    async fn upsert_branding(
        &self,
        ctx: ActingContext,
        cmd: UpsertBrandingCommand,
    ) -> Result<CompanyBranding, CompaniesError> {
        self.branding.upsert(&ctx, cmd).await
    }

    async fn get_safety_settings(
        &self,
        company_id: CompanyId,
    ) -> Result<SafetySettings, CompaniesError> {
        self.safety_settings.get(company_id).await
    }

    async fn upsert_safety_settings(
        &self,
        ctx: ActingContext,
        cmd: UpsertSafetySettingsCommand,
    ) -> Result<SafetySettings, CompaniesError> {
        self.safety_settings.upsert(&ctx, cmd).await
    }

    async fn get_regional_settings(
        &self,
        company_id: CompanyId,
    ) -> Result<RegionalSettings, CompaniesError> {
        self.regional_settings.get(company_id).await
    }

    async fn upsert_regional_settings(
        &self,
        ctx: ActingContext,
        cmd: UpsertRegionalSettingsCommand,
    ) -> Result<RegionalSettings, CompaniesError> {
        self.regional_settings.upsert(&ctx, cmd).await
    }

    async fn list_default_templates(
        &self,
        company_id: CompanyId,
    ) -> Result<Vec<DefaultTemplate>, CompaniesError> {
        self.templates.list(company_id).await
    }

    async fn upsert_default_template(
        &self,
        ctx: ActingContext,
        cmd: UpsertDefaultTemplateCommand,
    ) -> Result<DefaultTemplate, CompaniesError> {
        self.templates.upsert(&ctx, cmd).await
    }

    async fn get_notification_defaults(
        &self,
        company_id: CompanyId,
    ) -> Result<NotificationDefaults, CompaniesError> {
        self.notification_defaults.get(company_id).await
    }

    async fn upsert_notification_defaults(
        &self,
        ctx: ActingContext,
        cmd: UpsertNotificationDefaultsCommand,
    ) -> Result<NotificationDefaults, CompaniesError> {
        self.notification_defaults.upsert(&ctx, cmd).await
    }

    async fn get_storage_configuration(
        &self,
        company_id: CompanyId,
    ) -> Result<StorageConfiguration, CompaniesError> {
        self.storage.get(company_id).await
    }

    async fn upsert_storage_configuration(
        &self,
        ctx: ActingContext,
        cmd: UpsertStorageConfigurationCommand,
    ) -> Result<StorageConfiguration, CompaniesError> {
        self.storage.upsert(&ctx, cmd).await
    }
}
