//! PostgreSQL foundation helpers for Proven.
//!
//! Connection pooling, sqlx migrations, seed runner, health probes, and
//! database/migration version reporting. **No business tables.**

mod error;
mod health;
mod migrate;
mod pool;
mod seed;
mod version;

pub use error::DbError;
pub use health::{check_health, DbHealth};
pub use migrate::{migrate, migrations_dir, MigrationStatus};
pub use pool::{connect_pool, PostgresPool};
pub use seed::{run_seeds, seeds_dir};
pub use version::{database_version, DatabaseVersion};
