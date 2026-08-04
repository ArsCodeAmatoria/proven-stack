//! HTTP surface under `/api/v1/projects/*` (ADR-0009).

pub mod dto;
pub mod extractors;
pub mod handlers;
pub mod router;

pub use router::router;
