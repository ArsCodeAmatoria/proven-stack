//! Virus-scan hooks for FileApi complete path (ADR-0010).
//!
//! **Pending integration:** Go `media-worker` + Temporal `FileMediaProcessingWorkflow` perform
//! real AV scanning. Until wired, use [`PassthroughVirusScanHook`] (marks Clean immediately)
//! or [`EnqueuePendingVirusScanHook`] (leaves status Pending for a future worker callback).

use async_trait::async_trait;

use crate::application::ports::VirusScanHook;
use crate::domain::{CoreError, VirusScanOutcome, VirusScanRequest};

/// Marks every upload Clean immediately — suitable for unit tests and local smoke without workers.
#[derive(Debug, Default, Clone, Copy)]
pub struct PassthroughVirusScanHook;

#[async_trait]
impl VirusScanHook for PassthroughVirusScanHook {
    async fn scan(&self, _req: VirusScanRequest) -> Result<VirusScanOutcome, CoreError> {
        Ok(VirusScanOutcome::Clean {
            detail: Some("passthrough_hook".into()),
        })
    }
}

/// Records Pending and expects a media worker to call `FileApi::apply_scan_result` later.
#[derive(Debug, Default, Clone, Copy)]
pub struct EnqueuePendingVirusScanHook;

#[async_trait]
impl VirusScanHook for EnqueuePendingVirusScanHook {
    async fn scan(&self, req: VirusScanRequest) -> Result<VirusScanOutcome, CoreError> {
        tracing::info!(
            tenant_id = %req.tenant_id,
            file_id = %req.file_id,
            storage_key = %req.storage_key,
            "virus scan enqueued (media-worker integration pending)"
        );
        Ok(VirusScanOutcome::Pending {
            detail: Some("enqueued_pending_media_worker".into()),
        })
    }
}
