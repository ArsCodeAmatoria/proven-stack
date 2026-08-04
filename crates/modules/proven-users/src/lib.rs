//! Proven Users — account profile, kinds, preferences, signature prefs, emergency contacts
//! (ADR-0006).
//!
//! Core (`proven-core`) remains the System of Record for a user's **login identity**
//! (`UserId`, invite/activate/lock lifecycle, credentials, SSO links, sessions, AuthZ grants).
//! This module is the System of Record for a user's **account profile & preferences**, keyed by
//! that same `UserId` + `TenantId` — never the other way around. See `domain::ownership` for the
//! full boundary and non-goals (no project assignments, no People workforce SoR, no password
//! storage, no guest signing tokens).
//!
//! Other modules and Temporal activities must depend only on [`UsersApi`] — never on
//! `domain`/`infrastructure` internals or this module's Postgres schema directly (ADR-0006,
//! mirrors ADR-0001/ADR-0003 for Core and ADR-0005 for Companies).

pub mod api;
pub mod application;
pub mod domain;
pub mod events;
pub mod infrastructure;

use std::sync::Arc;

use axum::Router;
use proven_core::{AuthzApi, IdentityApi};

pub use application::services::ActingContext;
pub use application::{UsersApi, UsersPorts, UsersServices};
pub use domain::{
    AccessibilityPreferences, AuthenticationProfile, Avatar, DigestCadence,
    DigitalSignatureProfile, EmergencyContact, EmergencyContactId, LocalePreferences,
    NotificationPreferences, ProfileAuditEntry, ProfileAuditEntryId, ProfileStatus, SignatureType,
    UserKind, UserKindAssignment, UserKindAssignmentId, UserProfile, UserSetting, UsersError,
};
pub use events::UsersEvent;

/// Process-local handle to Users: shared services + the HTTP router builder.
///
/// ```
/// use proven_users::UsersModule;
///
/// let module = UsersModule::in_memory();
/// let _router = module.router();
/// ```
#[derive(Clone)]
pub struct UsersModule {
    pub services: Arc<UsersServices>,
}

impl UsersModule {
    /// Build a Users module backed entirely by the in-memory store, with a stub Allow-all AuthZ
    /// and no `IdentityApi` wired — no Core, no Postgres required. Used for unit tests and
    /// no-dependency local development.
    pub fn in_memory() -> Self {
        Self {
            services: Arc::new(UsersServices::in_memory_unchecked()),
        }
    }

    /// Build a Users module over in-memory ports, wired to a real `proven-core` `AuthzApi` +
    /// `IdentityApi` (any type implementing both — typically `proven_core::CoreServices`).
    pub fn with_core<C>(core: Arc<C>) -> Self
    where
        C: AuthzApi + IdentityApi + Send + Sync + 'static,
    {
        Self {
            services: Arc::new(UsersServices::with_core(UsersPorts::in_memory(), core)),
        }
    }

    /// Build a Users module from an arbitrary set of repository ports plus AuthZ/Identity.
    pub fn from_ports(
        ports: UsersPorts,
        authz: Arc<dyn AuthzApi>,
        identity: Option<Arc<dyn IdentityApi>>,
    ) -> Self {
        Self {
            services: Arc::new(UsersServices::new(ports, authz, identity)),
        }
    }

    /// Mount Users' HTTP surface under `/api/v1/users/*`. Callers merge this into the platform
    /// host router.
    pub fn router(self) -> Router {
        api::router(self)
    }
}
