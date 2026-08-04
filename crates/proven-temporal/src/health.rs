//! Temporal health checks (TCP probe + registry readiness).

use std::net::ToSocketAddrs;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::config::TemporalClientConfig;
use crate::error::TemporalError;
use crate::registry::{ActivityRegistry, WorkflowRegistry};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalHealthStatus {
    Healthy,
    Degraded,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalHealth {
    pub status: TemporalHealthStatus,
    pub address: String,
    pub namespace: String,
    pub reachable: bool,
    pub workflow_definitions: usize,
    pub activity_definitions: usize,
    pub detail: String,
}

/// Probes Temporal frontend reachability and reports registry sizes.
pub struct TemporalHealthChecker {
    config: TemporalClientConfig,
}

impl TemporalHealthChecker {
    pub fn new(config: TemporalClientConfig) -> Self {
        Self { config }
    }

    pub async fn check(
        &self,
        workflows: &WorkflowRegistry,
        activities: &ActivityRegistry,
    ) -> TemporalHealth {
        match probe_tcp(&self.config.address, self.config.connect_timeout_ms).await {
            Ok(()) => TemporalHealth {
                status: TemporalHealthStatus::Healthy,
                address: self.config.address.clone(),
                namespace: self.config.namespace.clone(),
                reachable: true,
                workflow_definitions: workflows.len(),
                activity_definitions: activities.len(),
                detail: if workflows.is_empty() && activities.is_empty() {
                    "reachable; infrastructure only (no workflows/activities registered)"
                        .into()
                } else {
                    "reachable".into()
                },
            },
            Err(err) => TemporalHealth {
                status: TemporalHealthStatus::Unavailable,
                address: self.config.address.clone(),
                namespace: self.config.namespace.clone(),
                reachable: false,
                workflow_definitions: workflows.len(),
                activity_definitions: activities.len(),
                detail: err.to_string(),
            },
        }
    }

    pub async fn ensure_reachable(&self) -> Result<(), TemporalError> {
        probe_tcp(&self.config.address, self.config.connect_timeout_ms)
            .await
            .map_err(|e| TemporalError::Connection(e.to_string()))
    }
}

pub async fn probe_tcp(address: &str, timeout_ms: u64) -> Result<(), TemporalError> {
    let mut addrs = address
        .to_socket_addrs()
        .map_err(|e| TemporalError::Connection(format!("resolve {address}: {e}")))?;
    let addr = addrs.next().ok_or_else(|| {
        TemporalError::Connection(format!("no addresses resolved for {address}"))
    })?;

    timeout(
        Duration::from_millis(timeout_ms),
        TcpStream::connect(addr),
    )
    .await
    .map_err(|_| TemporalError::Connection(format!("connect timed out for {address}")))?
    .map_err(|e| TemporalError::Connection(format!("connect {addr}: {e}")))?;
    Ok(())
}
