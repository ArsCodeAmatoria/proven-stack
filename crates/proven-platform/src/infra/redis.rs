use anyhow::Context;
use proven_config::Config;
use redis::aio::ConnectionManager;
use redis::Client;
use tokio::time::{timeout, Duration};

pub type RedisHandle = ConnectionManager;

pub async fn connect_redis(config: &Config) -> anyhow::Result<RedisHandle> {
    let url = config.redis.url.expose();
    let client = Client::open(url).context("redis client")?;
    let mut manager = timeout(Duration::from_secs(5), ConnectionManager::new(client))
        .await
        .context("redis connect timed out")?
        .context("redis connection manager")?;

    let pong: String = timeout(
        Duration::from_secs(3),
        redis::cmd("PING").query_async(&mut manager),
    )
    .await
    .context("redis ping timed out")?
    .context("redis ping")?;

    if pong != "PONG" {
        anyhow::bail!("unexpected redis PING response: {pong}");
    }

    Ok(manager)
}
