//! Infrastructure adapters (no business rules).

mod db;
mod nats;
mod redis;
mod temporal;

pub use db::{connect_postgres, PostgresPool};
pub use nats::{connect_nats, NatsHandle};
pub use redis::{connect_redis, RedisHandle};
pub use temporal::{connect_temporal, TemporalHandle};
