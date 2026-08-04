//! Enterprise RBAC engines (ADR-0007, AUTHORIZATION_RBAC_ARCHITECTURE.md). Pure domain logic —
//! `application::services::authz_service::AuthzService` is the only caller; `AuthzApi` remains
//! the sole decision authority (ADR-0003). See `docs/development/ENTERPRISE_RBAC.md`.

pub mod permission_engine;
pub mod policies;
pub mod role_engine;

pub use permission_engine::{scope_covers, EvaluationInput, PermissionEngine};
pub use policies::{AbacContext, AuthorizationPolicy, DefaultRbacPolicy, SealedResourcePolicy};
pub use role_engine::RoleEngine;
