//! Proven Companies — company profile, business units, addresses, contacts, branding, and
//! settings (ADR-0005).
//!
//! Core (`proven-core`) remains the System of Record for a Company's **legal identity**
//! (`legal_name`, `company_type`, lifecycle `status`) and mints the stable `CompanyId` every
//! module references. This module is the System of Record for a Company's **profile &
//! configuration**, keyed by that same `CompanyId` + `TenantId` — never the other way around.
//! See `domain::ownership` for the full boundary and non-goals.
//!
//! Other modules and Temporal activities must depend only on [`CompaniesApi`] — never on
//! `domain`/`infrastructure` internals or this module's Postgres schema directly (ADR-0005,
//! mirrors ADR-0001/ADR-0003 for Core).

pub mod api;
pub mod application;
pub mod domain;
pub mod events;
pub mod infrastructure;

use std::sync::Arc;

use axum::Router;
use proven_core::{AuthzApi, TenancyApi};

pub use application::{CompaniesApi, CompaniesPorts, CompaniesServices};
pub use application::services::ActingContext;
pub use domain::{
    Address, AddressId, AddressKind, BusinessUnit, BusinessUnitId, BusinessUnitStatus,
    CompaniesError, CompanyBranding, CompanyProfile, Contact, ContactId, ContactKind,
    DefaultTemplate, DefaultTemplateId, DigestCadence, MeasurementSystem, NotificationDefaults,
    ProfileStatus, RegionalSettings, SafetySettings, StorageConfiguration, TemplateKind,
};
pub use events::CompaniesEvent;

/// Process-local handle to Companies: shared services + the HTTP router builder.
///
/// ```
/// use proven_companies::CompaniesModule;
///
/// let module = CompaniesModule::in_memory();
/// let _router = module.router();
/// ```
#[derive(Clone)]
pub struct CompaniesModule {
    pub services: Arc<CompaniesServices>,
}

impl CompaniesModule {
    /// Build a Companies module backed entirely by the in-memory store, with a stub Allow-all
    /// AuthZ and no `TenancyApi` wired — no Core, no Postgres required. Used for unit tests and
    /// no-dependency local development.
    pub fn in_memory() -> Self {
        Self {
            services: Arc::new(CompaniesServices::in_memory_unchecked()),
        }
    }

    /// Build a Companies module over in-memory ports, wired to a real `proven-core` `AuthzApi` +
    /// `TenancyApi` (any type implementing both — typically `proven_core::CoreServices`).
    pub fn with_core<C>(core: Arc<C>) -> Self
    where
        C: AuthzApi + TenancyApi + Send + Sync + 'static,
    {
        Self {
            services: Arc::new(CompaniesServices::with_core(
                CompaniesPorts::in_memory(),
                core,
            )),
        }
    }

    /// Build a Companies module from an arbitrary set of repository ports plus AuthZ/Tenancy.
    pub fn from_ports(
        ports: CompaniesPorts,
        authz: Arc<dyn AuthzApi>,
        tenancy: Option<Arc<dyn TenancyApi>>,
    ) -> Self {
        Self {
            services: Arc::new(CompaniesServices::new(ports, authz, tenancy)),
        }
    }

    /// Mount Companies' HTTP surface under `/api/v1/companies/*`. Callers merge this into the
    /// platform host router.
    pub fn router(self) -> Router {
        api::router(self)
    }
}
