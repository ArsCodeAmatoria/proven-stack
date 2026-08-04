//! In-memory outbox event buffer. Production deployments would publish through the platform
//! outbox / NATS (mirrors `proven_core::infrastructure::outbox`); this adapter is for tests and
//! no-DB mode.

use std::sync::RwLock;

use async_trait::async_trait;

use crate::application::ports::EventPublisher;
use crate::domain::UsersError;
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

    /// Snapshot of every event published so far, in publish order (test inspection helper).
    pub fn events(&self) -> Result<Vec<EventEnvelope>, UsersError> {
        Ok(self
            .events
            .read()
            .map_err(|_| UsersError::Internal("outbox lock poisoned".into()))?
            .clone())
    }

    pub fn len(&self) -> Result<usize, UsersError> {
        Ok(self.events()?.len())
    }

    pub fn is_empty(&self) -> Result<bool, UsersError> {
        Ok(self.len()? == 0)
    }
}

#[async_trait]
impl EventPublisher for InMemoryOutbox {
    async fn publish(&self, envelope: EventEnvelope) -> Result<(), UsersError> {
        self.events
            .write()
            .map_err(|_| UsersError::Internal("outbox lock poisoned".into()))?
            .push(envelope);
        Ok(())
    }
}
