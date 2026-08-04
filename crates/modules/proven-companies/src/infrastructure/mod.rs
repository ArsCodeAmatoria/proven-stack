//! Infrastructure adapters implementing `application::ports` traits (ADR-0005). A Postgres
//! adapter against the `companies` schema is a follow-up; the in-memory store is authoritative
//! for now and is safe for production no-DB deployment modes.

pub mod memory;
pub mod outbox;

pub use memory::MemoryStore;
pub use outbox::InMemoryOutbox;
