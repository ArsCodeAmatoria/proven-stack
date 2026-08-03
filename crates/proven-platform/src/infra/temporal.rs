//! Temporal client handle (foundation).
//!
//! Full Temporal Rust SDK activity/workflow registration lands with workflows.
//! This handle stores connection settings and performs a TCP readiness probe.

use std::net::ToSocketAddrs;
use std::time::Duration;

use anyhow::{anyhow, Context};
use proven_config::Config;
use tokio::net::TcpStream;
use tokio::time::timeout;

#[derive(Clone, Debug)]
pub struct TemporalHandle {
    address: String,
    namespace: String,
}

impl TemporalHandle {
    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }
}

pub async fn connect_temporal(config: &Config) -> anyhow::Result<TemporalHandle> {
    let address = config.temporal.address.clone();
    let namespace = config.temporal.namespace.clone();

    probe_tcp(&address)
        .await
        .with_context(|| format!("temporal TCP probe failed for {address}"))?;

    Ok(TemporalHandle { address, namespace })
}

async fn probe_tcp(address: &str) -> anyhow::Result<()> {
    let mut addrs = address
        .to_socket_addrs()
        .with_context(|| format!("resolve temporal address {address}"))?;
    let addr = addrs
        .next()
        .ok_or_else(|| anyhow!("no addresses resolved for {address}"))?;

    timeout(Duration::from_secs(3), TcpStream::connect(addr))
        .await
        .map_err(|_| anyhow!("temporal connect timed out"))?
        .with_context(|| format!("connect {addr}"))?;
    Ok(())
}
