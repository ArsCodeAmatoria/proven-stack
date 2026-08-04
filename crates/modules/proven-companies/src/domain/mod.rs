//! Companies domain layer — pure types and rules, no I/O (ADR-0005).

mod enums;
mod error;
mod ids;
mod models;
pub mod ownership;
pub mod permissions;
pub mod validation;

pub use enums::{
    AddressKind, BusinessUnitStatus, ContactKind, DigestCadence, MeasurementSystem, ProfileStatus,
    TemplateKind,
};
pub use error::CompaniesError;
pub use ids::{AddressId, BusinessUnitId, ContactId, DefaultTemplateId};
pub use models::{
    Address, BusinessUnit, CompanyBranding, CompanyProfile, Contact, DefaultTemplate,
    NotificationDefaults, RegionalSettings, SafetySettings, StorageConfiguration,
};
