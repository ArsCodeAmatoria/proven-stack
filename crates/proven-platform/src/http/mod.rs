//! HTTP surface: router, health, middleware, error mapping.

pub mod db;
mod docs;
mod error;
pub mod health;
mod metrics;
mod middleware;
mod router;

pub use error::ApiError;
pub use middleware::CorrelationId;
pub use router::build_router;
