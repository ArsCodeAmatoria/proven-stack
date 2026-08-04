//! Full in-memory store implementing every repository port. Used for unit tests and any
//! no-Postgres deployment mode (mirrors `proven_core::infrastructure::memory` — ADR-0005 has no
//! SQL adapter yet; the `companies` schema migration + `infrastructure::postgres` land in a
//! follow-up).

use std::collections::HashMap;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use async_trait::async_trait;
use proven_shared::CompanyId;
use uuid::Uuid;

use crate::application::ports::{
    AddressRepository, BrandingRepository, BusinessUnitRepository, CompanyProfileRepository,
    ContactRepository, DefaultTemplateRepository, NotificationDefaultsRepository,
    RegionalSettingsRepository, SafetySettingsRepository, StorageConfigurationRepository,
};
use crate::domain::{
    Address, AddressId, BusinessUnit, BusinessUnitId, CompaniesError, CompanyBranding,
    CompanyProfile, Contact, ContactId, DefaultTemplate, NotificationDefaults, RegionalSettings,
    SafetySettings, StorageConfiguration, TemplateKind,
};

#[derive(Default)]
struct MemoryState {
    profiles: HashMap<Uuid, CompanyProfile>,
    business_units: HashMap<Uuid, BusinessUnit>,
    addresses: HashMap<Uuid, Address>,
    contacts: HashMap<Uuid, Contact>,
    branding: HashMap<Uuid, CompanyBranding>,
    safety_settings: HashMap<Uuid, SafetySettings>,
    regional_settings: HashMap<Uuid, RegionalSettings>,
    default_templates: HashMap<Uuid, DefaultTemplate>,
    notification_defaults: HashMap<Uuid, NotificationDefaults>,
    storage_configuration: HashMap<Uuid, StorageConfiguration>,
}

/// Shared, thread-safe in-memory backing store for every Companies port.
#[derive(Default)]
pub struct MemoryStore {
    state: RwLock<MemoryState>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(MemoryState::default()),
        }
    }

    fn read(&self) -> Result<RwLockReadGuard<'_, MemoryState>, CompaniesError> {
        self.state
            .read()
            .map_err(|_| CompaniesError::Internal("memory store lock poisoned".into()))
    }

    fn write(&self) -> Result<RwLockWriteGuard<'_, MemoryState>, CompaniesError> {
        self.state
            .write()
            .map_err(|_| CompaniesError::Internal("memory store lock poisoned".into()))
    }
}

#[async_trait]
impl CompanyProfileRepository for MemoryStore {
    async fn get(&self, company_id: CompanyId) -> Result<Option<CompanyProfile>, CompaniesError> {
        Ok(self.read()?.profiles.get(&company_id.as_uuid()).cloned())
    }

    async fn upsert(&self, profile: &CompanyProfile) -> Result<(), CompaniesError> {
        self.write()?
            .profiles
            .insert(profile.company_id.as_uuid(), profile.clone());
        Ok(())
    }
}

#[async_trait]
impl BusinessUnitRepository for MemoryStore {
    async fn insert(&self, unit: &BusinessUnit) -> Result<(), CompaniesError> {
        self.write()?
            .business_units
            .insert(unit.id.as_uuid(), unit.clone());
        Ok(())
    }

    async fn get(
        &self,
        company_id: CompanyId,
        id: BusinessUnitId,
    ) -> Result<Option<BusinessUnit>, CompaniesError> {
        Ok(self
            .read()?
            .business_units
            .get(&id.as_uuid())
            .filter(|u| u.company_id == company_id)
            .cloned())
    }

    async fn list(&self, company_id: CompanyId) -> Result<Vec<BusinessUnit>, CompaniesError> {
        Ok(self
            .read()?
            .business_units
            .values()
            .filter(|u| u.company_id == company_id)
            .cloned()
            .collect())
    }

    async fn update(&self, unit: &BusinessUnit) -> Result<(), CompaniesError> {
        self.write()?
            .business_units
            .insert(unit.id.as_uuid(), unit.clone());
        Ok(())
    }
}

#[async_trait]
impl AddressRepository for MemoryStore {
    async fn insert(&self, address: &Address) -> Result<(), CompaniesError> {
        self.write()?
            .addresses
            .insert(address.id.as_uuid(), address.clone());
        Ok(())
    }

    async fn get(
        &self,
        company_id: CompanyId,
        id: AddressId,
    ) -> Result<Option<Address>, CompaniesError> {
        Ok(self
            .read()?
            .addresses
            .get(&id.as_uuid())
            .filter(|a| a.company_id == company_id)
            .cloned())
    }

    async fn list(&self, company_id: CompanyId) -> Result<Vec<Address>, CompaniesError> {
        Ok(self
            .read()?
            .addresses
            .values()
            .filter(|a| a.company_id == company_id)
            .cloned()
            .collect())
    }

    async fn update(&self, address: &Address) -> Result<(), CompaniesError> {
        self.write()?
            .addresses
            .insert(address.id.as_uuid(), address.clone());
        Ok(())
    }

    async fn remove(&self, company_id: CompanyId, id: AddressId) -> Result<(), CompaniesError> {
        let mut state = self.write()?;
        match state.addresses.get(&id.as_uuid()) {
            Some(a) if a.company_id == company_id => {
                state.addresses.remove(&id.as_uuid());
                Ok(())
            }
            _ => Err(CompaniesError::NotFound("address")),
        }
    }
}

#[async_trait]
impl ContactRepository for MemoryStore {
    async fn insert(&self, contact: &Contact) -> Result<(), CompaniesError> {
        self.write()?
            .contacts
            .insert(contact.id.as_uuid(), contact.clone());
        Ok(())
    }

    async fn get(
        &self,
        company_id: CompanyId,
        id: ContactId,
    ) -> Result<Option<Contact>, CompaniesError> {
        Ok(self
            .read()?
            .contacts
            .get(&id.as_uuid())
            .filter(|c| c.company_id == company_id)
            .cloned())
    }

    async fn list(&self, company_id: CompanyId) -> Result<Vec<Contact>, CompaniesError> {
        Ok(self
            .read()?
            .contacts
            .values()
            .filter(|c| c.company_id == company_id)
            .cloned()
            .collect())
    }

    async fn update(&self, contact: &Contact) -> Result<(), CompaniesError> {
        self.write()?
            .contacts
            .insert(contact.id.as_uuid(), contact.clone());
        Ok(())
    }

    async fn remove(&self, company_id: CompanyId, id: ContactId) -> Result<(), CompaniesError> {
        let mut state = self.write()?;
        match state.contacts.get(&id.as_uuid()) {
            Some(c) if c.company_id == company_id => {
                state.contacts.remove(&id.as_uuid());
                Ok(())
            }
            _ => Err(CompaniesError::NotFound("contact")),
        }
    }
}

#[async_trait]
impl BrandingRepository for MemoryStore {
    async fn get(&self, company_id: CompanyId) -> Result<Option<CompanyBranding>, CompaniesError> {
        Ok(self.read()?.branding.get(&company_id.as_uuid()).cloned())
    }

    async fn upsert(&self, branding: &CompanyBranding) -> Result<(), CompaniesError> {
        self.write()?
            .branding
            .insert(branding.company_id.as_uuid(), branding.clone());
        Ok(())
    }
}

#[async_trait]
impl SafetySettingsRepository for MemoryStore {
    async fn get(&self, company_id: CompanyId) -> Result<Option<SafetySettings>, CompaniesError> {
        Ok(self
            .read()?
            .safety_settings
            .get(&company_id.as_uuid())
            .cloned())
    }

    async fn upsert(&self, settings: &SafetySettings) -> Result<(), CompaniesError> {
        self.write()?
            .safety_settings
            .insert(settings.company_id.as_uuid(), settings.clone());
        Ok(())
    }
}

#[async_trait]
impl RegionalSettingsRepository for MemoryStore {
    async fn get(
        &self,
        company_id: CompanyId,
    ) -> Result<Option<RegionalSettings>, CompaniesError> {
        Ok(self
            .read()?
            .regional_settings
            .get(&company_id.as_uuid())
            .cloned())
    }

    async fn upsert(&self, settings: &RegionalSettings) -> Result<(), CompaniesError> {
        self.write()?
            .regional_settings
            .insert(settings.company_id.as_uuid(), settings.clone());
        Ok(())
    }
}

#[async_trait]
impl DefaultTemplateRepository for MemoryStore {
    async fn list(&self, company_id: CompanyId) -> Result<Vec<DefaultTemplate>, CompaniesError> {
        Ok(self
            .read()?
            .default_templates
            .values()
            .filter(|t| t.company_id == company_id)
            .cloned()
            .collect())
    }

    async fn get_by_kind(
        &self,
        company_id: CompanyId,
        kind: TemplateKind,
    ) -> Result<Option<DefaultTemplate>, CompaniesError> {
        Ok(self
            .read()?
            .default_templates
            .values()
            .find(|t| t.company_id == company_id && t.kind == kind)
            .cloned())
    }

    async fn upsert(&self, template: &DefaultTemplate) -> Result<(), CompaniesError> {
        let mut state = self.write()?;
        let existing_id = state
            .default_templates
            .values()
            .find(|t| t.company_id == template.company_id && t.kind == template.kind)
            .map(|t| t.id.as_uuid());
        if let Some(id) = existing_id {
            if id != template.id.as_uuid() {
                state.default_templates.remove(&id);
            }
        }
        state
            .default_templates
            .insert(template.id.as_uuid(), template.clone());
        Ok(())
    }
}

#[async_trait]
impl NotificationDefaultsRepository for MemoryStore {
    async fn get(
        &self,
        company_id: CompanyId,
    ) -> Result<Option<NotificationDefaults>, CompaniesError> {
        Ok(self
            .read()?
            .notification_defaults
            .get(&company_id.as_uuid())
            .cloned())
    }

    async fn upsert(&self, defaults: &NotificationDefaults) -> Result<(), CompaniesError> {
        self.write()?
            .notification_defaults
            .insert(defaults.company_id.as_uuid(), defaults.clone());
        Ok(())
    }
}

#[async_trait]
impl StorageConfigurationRepository for MemoryStore {
    async fn get(
        &self,
        company_id: CompanyId,
    ) -> Result<Option<StorageConfiguration>, CompaniesError> {
        Ok(self
            .read()?
            .storage_configuration
            .get(&company_id.as_uuid())
            .cloned())
    }

    async fn upsert(&self, config: &StorageConfiguration) -> Result<(), CompaniesError> {
        self.write()?
            .storage_configuration
            .insert(config.company_id.as_uuid(), config.clone());
        Ok(())
    }
}
