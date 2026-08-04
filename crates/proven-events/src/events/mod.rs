//! Initial integration events published on NATS (ADR-0011).
//!
//! These are the first shared catalog events. Module-local domain events remain in each
//! `proven-*` crate; adapters may map domain facts onto these integration shapes.

mod payloads;

pub use payloads::{
    AuditRecorded, CompanyCreated, FileUploaded, InitialEvent, ProjectCreated, UserCreated,
};
pub use payloads::subjects;
