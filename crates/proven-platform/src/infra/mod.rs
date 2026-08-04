//! Infrastructure adapters (no business rules).

mod db;
mod events;
mod nats;
mod redis;
mod temporal;

pub use db::{connect_postgres, PostgresPool};
pub use events::{
    event_publisher, event_publisher_with_options, event_subscriber, event_subscriber_with_options,
};
pub use nats::{connect_nats, NatsHandle};
pub use redis::{connect_redis, RedisHandle};
pub use temporal::{connect_temporal, TemporalHandle};
