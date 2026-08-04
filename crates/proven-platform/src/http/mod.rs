//! HTTP surface: router, health, middleware, error mapping.

pub mod db;
mod docs;
mod error;
pub mod health;
mod metrics;
mod middleware;
mod router;
pub mod temporal;

pub use error::ApiError;
pub use middleware::{
    require_permission, AuthnPolicy, AuthzPrincipal, CorrelationId, RateLimitState,
};
pub use router::build_router;
