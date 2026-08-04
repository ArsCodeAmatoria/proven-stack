//! HTTP surface for Users (ADR-0006 §3). Thin transport layer only — no business rules live
//! here.

pub mod dto;
pub mod extractors;
pub mod handlers;
pub mod router;

pub use router::router;
