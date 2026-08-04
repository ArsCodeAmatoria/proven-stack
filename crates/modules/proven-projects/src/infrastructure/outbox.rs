//! In-memory outbox for Projects events.

use std::sync::RwLock;

use async_trait::async_trait;

use crate::application::ports::EventPublisher;
use crate::domain::ProjectsError;
use crate::events::EventEnvelope;

#[derive(Default)]
pub struct InMemoryOutbox {
    events: RwLock<Vec<EventEnvelope>>,
}

impl InMemoryOutbox {
    pub fn new() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl EventPublisher for InMemoryOutbox {
    async fn publish(&self, envelope: EventEnvelope) -> Result<(), ProjectsError> {
        self.events
            .write()
            .map_err(|_| ProjectsError::Internal("outbox lock poisoned".into()))?
            .push(envelope);
        Ok(())
    }
}
