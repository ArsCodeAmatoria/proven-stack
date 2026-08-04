//! Repository / outbound ports. Implemented by `infrastructure::memory` (always) and, in a
//! future iteration, `infrastructure::postgres` against the `companies` schema. Application
//! services depend only on these traits — never on a concrete storage engine (ADR-0005, mirrors
//! ADR-0004 for Core).

use async_trait::async_trait;

use proven_shared::CompanyId;

use crate::domain::{
    Address, AddressId, BusinessUnit, BusinessUnitId, CompaniesError, CompanyBranding,
    CompanyProfile, Contact, ContactId, DefaultTemplate, NotificationDefaults, RegionalSettings,
    SafetySettings, StorageConfiguration, TemplateKind,
};
use crate::events::EventEnvelope;

#[async_trait]
pub trait CompanyProfileRepository: Send + Sync {
    async fn get(&self, company_id: CompanyId) -> Result<Option<CompanyProfile>, CompaniesError>;
    async fn upsert(&self, profile: &CompanyProfile) -> Result<(), CompaniesError>;
}

#[async_trait]
pub trait BusinessUnitRepository: Send + Sync {
    async fn insert(&self, unit: &BusinessUnit) -> Result<(), CompaniesError>;
    async fn get(
        &self,
        company_id: CompanyId,
        id: BusinessUnitId,
    ) -> Result<Option<BusinessUnit>, CompaniesError>;
    async fn list(&self, company_id: CompanyId) -> Result<Vec<BusinessUnit>, CompaniesError>;
    async fn update(&self, unit: &BusinessUnit) -> Result<(), CompaniesError>;
}

#[async_trait]
pub trait AddressRepository: Send + Sync {
    async fn insert(&self, address: &Address) -> Result<(), CompaniesError>;
    async fn get(
        &self,
        company_id: CompanyId,
        id: AddressId,
    ) -> Result<Option<Address>, CompaniesError>;
    async fn list(&self, company_id: CompanyId) -> Result<Vec<Address>, CompaniesError>;
    async fn update(&self, address: &Address) -> Result<(), CompaniesError>;
    async fn remove(&self, company_id: CompanyId, id: AddressId) -> Result<(), CompaniesError>;
}

#[async_trait]
pub trait ContactRepository: Send + Sync {
    async fn insert(&self, contact: &Contact) -> Result<(), CompaniesError>;
    async fn get(
        &self,
        company_id: CompanyId,
        id: ContactId,
    ) -> Result<Option<Contact>, CompaniesError>;
    async fn list(&self, company_id: CompanyId) -> Result<Vec<Contact>, CompaniesError>;
    async fn update(&self, contact: &Contact) -> Result<(), CompaniesError>;
    async fn remove(&self, company_id: CompanyId, id: ContactId) -> Result<(), CompaniesError>;
}

#[async_trait]
pub trait BrandingRepository: Send + Sync {
    async fn get(&self, company_id: CompanyId) -> Result<Option<CompanyBranding>, CompaniesError>;
    async fn upsert(&self, branding: &CompanyBranding) -> Result<(), CompaniesError>;
}

#[async_trait]
pub trait SafetySettingsRepository: Send + Sync {
    async fn get(&self, company_id: CompanyId) -> Result<Option<SafetySettings>, CompaniesError>;
    async fn upsert(&self, settings: &SafetySettings) -> Result<(), CompaniesError>;
}

#[async_trait]
pub trait RegionalSettingsRepository: Send + Sync {
    async fn get(
        &self,
        company_id: CompanyId,
    ) -> Result<Option<RegionalSettings>, CompaniesError>;
    async fn upsert(&self, settings: &RegionalSettings) -> Result<(), CompaniesError>;
}

#[async_trait]
pub trait DefaultTemplateRepository: Send + Sync {
    async fn list(&self, company_id: CompanyId) -> Result<Vec<DefaultTemplate>, CompaniesError>;
    async fn get_by_kind(
        &self,
        company_id: CompanyId,
        kind: TemplateKind,
    ) -> Result<Option<DefaultTemplate>, CompaniesError>;
    async fn upsert(&self, template: &DefaultTemplate) -> Result<(), CompaniesError>;
}

#[async_trait]
pub trait NotificationDefaultsRepository: Send + Sync {
    async fn get(
        &self,
        company_id: CompanyId,
    ) -> Result<Option<NotificationDefaults>, CompaniesError>;
    async fn upsert(&self, defaults: &NotificationDefaults) -> Result<(), CompaniesError>;
}

#[async_trait]
pub trait StorageConfigurationRepository: Send + Sync {
    async fn get(
        &self,
        company_id: CompanyId,
    ) -> Result<Option<StorageConfiguration>, CompaniesError>;
    async fn upsert(&self, config: &StorageConfiguration) -> Result<(), CompaniesError>;
}

/// Outbound event transport (in-memory buffer for tests; NATS/outbox in production, mirroring
/// `proven_core::application::ports::EventPublisher`).
#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, envelope: EventEnvelope) -> Result<(), CompaniesError>;
}
