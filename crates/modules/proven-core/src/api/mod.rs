//! HTTP surface for Core (CORE_DOMAIN.md §13.2). Thin transport layer only — no business
//! rules live here.

pub mod dto;
pub mod extractors;
pub mod handlers;
pub mod router;

pub use router::router;
