//! Event publishers — NATS + in-memory (tests).

use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use async_nats::Client;
use bytes::Bytes;
use tracing::{info, instrument};

use crate::envelope::EventEnvelope;
use crate::error::EventError;
use crate::retry::{retry_with_backoff, RetryPolicy};

/// Options applied when publishing.
#[derive(Debug, Clone, Default)]
pub struct PublishOptions {
    pub retry: RetryPolicy,
}

#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, envelope: EventEnvelope) -> Result<EventEnvelope, EventError>;
}

/// Publishes envelopes to NATS core subjects with retry + structured logging.
pub struct NatsEventPublisher {
    client: Client,
    options: PublishOptions,
}

impl NatsEventPublisher {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            options: PublishOptions::default(),
        }
    }

    pub fn with_options(client: Client, options: PublishOptions) -> Self {
        Self { client, options }
    }
}

#[async_trait]
impl EventPublisher for NatsEventPublisher {
    #[instrument(
        name = "nats.publish",
        skip(self, envelope),
        fields(
            event_id = %envelope.event_id,
            event_name = %envelope.event_name,
            subject = tracing::field::Empty,
            tenant_id = %envelope.tenant_id,
        )
    )]
    async fn publish(&self, envelope: EventEnvelope) -> Result<EventEnvelope, EventError> {
        let mut envelope = envelope.mark_published();
        let subject = envelope.subject();
        tracing::Span::current().record("subject", subject.as_str());

        let payload = envelope.to_bytes()?;
        let client = self.client.clone();
        let subject_clone = subject.clone();
        let bytes = Bytes::from(payload);

        retry_with_backoff(&self.options.retry, "nats_publish", |attempt| {
            let client = client.clone();
            let subject = subject_clone.clone();
            let bytes = bytes.clone();
            async move {
                client
                    .publish(subject.clone(), bytes)
                    .await
                    .map_err(|err| EventError::Publish(format!("attempt {attempt}: {err}")))?;
                // Ensure the message is flushed toward the server.
                client
                    .flush()
                    .await
                    .map_err(|err| EventError::Publish(format!("flush attempt {attempt}: {err}")))?;
                Ok(())
            }
        })
        .await?;

        info!(
            event_id = %envelope.event_id,
            event_name = %envelope.event_name,
            subject = %subject,
            event_version = %envelope.event_version,
            "event published to NATS"
        );

        // published_at already set; refresh in case retry delayed significantly
        envelope.published_at = Some(chrono::Utc::now());
        Ok(envelope)
    }
}

/// In-memory publisher for unit tests — records every published envelope.
#[derive(Default)]
pub struct InMemoryEventPublisher {
    events: RwLock<Vec<EventEnvelope>>,
}

impl InMemoryEventPublisher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn published(&self) -> Result<Vec<EventEnvelope>, EventError> {
        self.events
            .read()
            .map(|g| g.clone())
            .map_err(|_| EventError::Internal("publisher lock poisoned".into()))
    }

    pub fn len(&self) -> Result<usize, EventError> {
        Ok(self.published()?.len())
    }

    pub fn is_empty(&self) -> Result<bool, EventError> {
        Ok(self.len()? == 0)
    }

    pub fn shared(self) -> Arc<Self> {
        Arc::new(self)
    }
}

#[async_trait]
impl EventPublisher for InMemoryEventPublisher {
    async fn publish(&self, envelope: EventEnvelope) -> Result<EventEnvelope, EventError> {
        let envelope = envelope.mark_published();
        info!(
            event_id = %envelope.event_id,
            event_name = %envelope.event_name,
            subject = %envelope.subject(),
            "event published to in-memory bus"
        );
        self.events
            .write()
            .map_err(|_| EventError::Internal("publisher lock poisoned".into()))?
            .push(envelope.clone());
        Ok(envelope)
    }
}

#[async_trait]
impl EventPublisher for Arc<InMemoryEventPublisher> {
    async fn publish(&self, envelope: EventEnvelope) -> Result<EventEnvelope, EventError> {
        (**self).publish(envelope).await
    }
}
