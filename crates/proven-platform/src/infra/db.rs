use proven_config::Config;
use proven_db::connect_pool;

pub use proven_db::PostgresPool;

pub async fn connect_postgres(config: &Config) -> anyhow::Result<PostgresPool> {
    Ok(connect_pool(config).await?)
}
