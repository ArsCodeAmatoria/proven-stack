//! HTTP surface for Companies (ADR-0005 §4). Thin transport layer only — no business rules
//! live here.

pub mod dto;
pub mod extractors;
pub mod handlers;
pub mod router;

pub use router::router;
