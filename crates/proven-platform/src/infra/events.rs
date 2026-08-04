//! NATS event bus adapters for the platform host (ADR-0011).

use proven_events::{NatsEventPublisher, NatsEventSubscriber, PublishOptions, SubscribeOptions};

use super::NatsHandle;

/// Build a retrying NATS publisher from the connected client.
pub fn event_publisher(client: &NatsHandle) -> NatsEventPublisher {
    NatsEventPublisher::new(client.clone())
}

pub fn event_publisher_with_options(
    client: &NatsHandle,
    options: PublishOptions,
) -> NatsEventPublisher {
    NatsEventPublisher::with_options(client.clone(), options)
}

/// Build a NATS subscriber from the connected client.
pub fn event_subscriber(client: &NatsHandle) -> NatsEventSubscriber {
    NatsEventSubscriber::new(client.clone())
}

pub fn event_subscriber_with_options(
    client: &NatsHandle,
    options: SubscribeOptions,
) -> NatsEventSubscriber {
    NatsEventSubscriber::with_options(client.clone(), options)
}
