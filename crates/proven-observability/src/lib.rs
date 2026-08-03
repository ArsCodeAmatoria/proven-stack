//! Observability infrastructure for Proven (logs, traces, metrics, correlation).
//!
//! No dashboards — exporters and hooks only.

mod correlation;
mod init;
mod metrics;
mod otel;

pub use correlation::{
    CORRELATION_ID_HEADER, REQUEST_ID_HEADER, correlation_id_from_headers, ensure_correlation_id,
};
pub use init::{ObservabilityHandle, init_observability};
pub use metrics::{PrometheusHandle, install_metrics, render_metrics};
