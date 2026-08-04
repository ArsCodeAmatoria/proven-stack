//! Workflow and activity registries (empty until workflows land).

mod activity;
mod workflow;

pub use activity::{ActivityDefinition, ActivityRegistry};
pub use workflow::{WorkflowDefinition, WorkflowRegistry};
