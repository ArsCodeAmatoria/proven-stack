use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("database connection failed: {0}")]
    Connect(String),
    #[error("migration failed: {0}")]
    Migrate(String),
    #[error("seed failed: {0}")]
    Seed(String),
    #[error("health check failed: {0}")]
    Health(String),
    #[error("version query failed: {0}")]
    Version(String),
    #[error("{0}")]
    Other(String),
}
