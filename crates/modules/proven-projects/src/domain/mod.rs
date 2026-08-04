//! Projects domain layer — pure types and rules, no I/O (ADR-0009).

mod enums;
mod error;
mod ids;
mod models;
pub mod ownership;
pub mod permissions;
pub mod validation;

pub use enums::{ParticipantStatus, ParticipationRole, ProjectStatus};
pub use error::ProjectsError;
pub use ids::ParticipantId;
pub use models::{Project, ProjectLocation, ProjectParticipant};
