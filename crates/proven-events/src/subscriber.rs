//! Event subscribers — NATS + in-memory bus (tests).

use std::sync::{Arc, RwLock};

use async_nats::Client;
use async_trait::async_trait;
use futures_util::StreamExt;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

use crate::envelope::EventEnvelope;
use crate::error::EventError;
use crate::retry::{retry_with_backoff, RetryPolicy};

/// Async handler invoked for each received envelope.
#[async_trait]
pub trait EventHandler: Send + Sync + 'static {
    async fn handle(&self, envelope: EventEnvelope) -> Result<(), EventError>;
}

/// Closure adapter for simple handlers.
pub struct FnHandler<F>(pub F);

#[async_trait]
impl<F, Fut> EventHandler for FnHandler<F>
where
    F: Fn(EventEnvelope) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<(), EventError>> + Send + 'static,
{
    async fn handle(&self, envelope: EventEnvelope) -> Result<(), EventError> {
        (self.0)(envelope).await
    }
}

#[derive(Debug, Clone)]
pub struct SubscribeOptions {
    pub retry: RetryPolicy,
    /// When true, handler failures after retries are logged and skipped (at-most-once).
    pub ack_on_handler_failure: bool,
}

impl Default for SubscribeOptions {
    fn default() -> Self {
        Self {
            retry: RetryPolicy::default(),
            ack_on_handler_failure: true,
        }
    }
}

/// Handle that stops a background subscription when dropped/aborted.
pub struct SubscriptionHandle {
    abort: Option<JoinHandle<()>>,
}

impl SubscriptionHandle {
    pub fn abort(mut self) {
        if let Some(handle) = self.abort.take() {
            handle.abort();
        }
    }
}

impl Drop for SubscriptionHandle {
    fn drop(&mut self) {
        if let Some(handle) = self.abort.take() {
            handle.abort();
        }
    }
}

/// NATS core subscriber. JetStream durable consumers are a follow-up.
pub struct NatsEventSubscriber {
    client: Client,
    options: SubscribeOptions,
}

impl NatsEventSubscriber {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            options: SubscribeOptions::default(),
        }
    }

    pub fn with_options(client: Client, options: SubscribeOptions) -> Self {
        Self { client, options }
    }

    /// Subscribe to a subject or wildcard (e.g. `proven.core.v1.*`).
    pub async fn subscribe<H>(
        &self,
        subject: impl Into<String>,
        handler: H,
    ) -> Result<SubscriptionHandle, EventError>
    where
        H: EventHandler,
    {
        self.subscribe_handler(subject, Arc::new(handler)).await
    }

    pub async fn subscribe_handler(
        &self,
        subject: impl Into<String>,
        handler: Arc<dyn EventHandler>,
    ) -> Result<SubscriptionHandle, EventError> {
        let subject = subject.into();
        let mut sub = self
            .client
            .subscribe(subject.clone())
            .await
            .map_err(|err| EventError::Subscribe(err.to_string()))?;

        info!(subject = %subject, "NATS subscription started");

        let options = self.options.clone();
        let abort = tokio::spawn(async move {
            while let Some(msg) = sub.next().await {
                match EventEnvelope::from_bytes(&msg.payload) {
                    Ok(envelope) => {
                        let event_id = envelope.event_id;
                        let event_name = envelope.event_name.clone();
                        let result =
                            retry_with_backoff(&options.retry, "event_handler", |_| {
                                let envelope = envelope.clone();
                                let handler = handler.clone();
                                async move { handler.handle(envelope).await }
                            })
                            .await;

                        match result {
                            Ok(()) => {
                                info!(
                                    event_id = %event_id,
                                    event_name = %event_name,
                                    "event handled"
                                );
                            }
                            Err(err) => {
                                error!(
                                    error = %err,
                                    event_id = %event_id,
                                    ack_on_failure = options.ack_on_handler_failure,
                                    "event handler failed after retries"
                                );
                            }
                        }
                    }
                    Err(err) => {
                        warn!(error = %err, "failed to decode event envelope; dropping message");
                    }
                }
            }
            info!(subject = %subject, "NATS subscription ended");
        });

        Ok(SubscriptionHandle { abort: Some(abort) })
    }
}

/// In-memory fan-out bus for tests (no NATS required).
pub struct InMemoryEventBus {
    tx: broadcast::Sender<EventEnvelope>,
    published: RwLock<Vec<EventEnvelope>>,
}

impl InMemoryEventBus {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity.max(16));
        Self {
            tx,
            published: RwLock::new(Vec::new()),
        }
    }

    pub fn published(&self) -> Result<Vec<EventEnvelope>, EventError> {
        self.published
            .read()
            .map(|g| g.clone())
            .map_err(|_| EventError::Internal("bus lock poisoned".into()))
    }

    pub async fn publish(&self, envelope: EventEnvelope) -> Result<EventEnvelope, EventError> {
        let envelope = envelope.mark_published();
        self.published
            .write()
            .map_err(|_| EventError::Internal("bus lock poisoned".into()))?
            .push(envelope.clone());
        let _ = self.tx.send(envelope.clone());
        Ok(envelope)
    }

    pub async fn subscribe<H>(
        &self,
        subject_filter: impl Into<String>,
        handler: H,
        options: SubscribeOptions,
    ) -> SubscriptionHandle
    where
        H: EventHandler,
    {
        let filter = subject_filter.into();
        let mut rx = self.tx.subscribe();
        let handler = Arc::new(handler);
        let abort = tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(envelope) => {
                        let subject = envelope.subject();
                        if !subject_matches(&filter, &subject) {
                            continue;
                        }
                        let result =
                            retry_with_backoff(&options.retry, "event_handler", |_| {
                                let envelope = envelope.clone();
                                let handler = handler.clone();
                                async move { handler.handle(envelope).await }
                            })
                            .await;
                        if let Err(err) = result {
                            error!(error = %err, "in-memory handler failed");
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                }
            }
        });
        SubscriptionHandle { abort: Some(abort) }
    }
}

fn subject_matches(filter: &str, subject: &str) -> bool {
    if filter == subject {
        return true;
    }
    if let Some(prefix) = filter.strip_suffix(".*") {
        return subject == prefix || subject.starts_with(&format!("{prefix}."));
    }
    false
}
