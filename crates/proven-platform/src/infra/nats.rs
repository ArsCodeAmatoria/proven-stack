use anyhow::Context;
use proven_config::Config;
use tokio::time::{timeout, Duration};

pub type NatsHandle = async_nats::Client;

pub async fn connect_nats(config: &Config) -> anyhow::Result<NatsHandle> {
    let client = timeout(
        Duration::from_secs(5),
        async_nats::connect(config.nats.url.as_str()),
    )
    .await
    .context("nats connect timed out")?
    .context("nats connect")?;
    Ok(client)
}
