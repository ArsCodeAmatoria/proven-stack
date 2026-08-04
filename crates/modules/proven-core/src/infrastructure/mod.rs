//! Infrastructure adapters implementing `application::ports` traits (ADR-0004, ADR-0010).

pub mod memory;
pub mod object_storage;
pub mod outbox;
pub mod postgres;
pub mod virus_scan;

pub use memory::MemoryStore;
pub use object_storage::{PendingR2ObjectStorage, PlaceholderObjectStorage, R2StorageConfig};
pub use outbox::InMemoryOutbox;
pub use virus_scan::{EnqueuePendingVirusScanHook, PassthroughVirusScanHook};
