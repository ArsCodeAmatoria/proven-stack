//! Prometheus metrics recorder (scrape via `/metrics`).

use anyhow::Context;
use metrics_exporter_prometheus::PrometheusBuilder;

pub use metrics_exporter_prometheus::PrometheusHandle;

/// Install the global Prometheus recorder. Call once at process start.
pub fn install_metrics() -> anyhow::Result<PrometheusHandle> {
    PrometheusBuilder::new()
        .install_recorder()
        .context("failed to install prometheus metrics recorder")
}

pub fn render_metrics(handle: &PrometheusHandle) -> String {
    handle.render()
}
