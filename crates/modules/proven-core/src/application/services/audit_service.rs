//! `AuditEngine` — Core's append-only, integrity-digested audit Source of Record (ADR-0008,
//! CORE_DOMAIN.md §18, AUDIT_LOGGING_ARCHITECTURE.md). `AuditService` is the concrete type;
//! [`AuditEngine`] is a type alias — new code should refer to `AuditEngine`.
//!
//! Hard rules enforced here: append-only (no update/delete method exists on this service), no
//! secrets in payloads (caller responsibility — this service never inspects payload contents),
//! and every `AppendAuditEntryCommand` capture field beyond the original five is optional with a
//! sensible default so pre-ADR-0008 callers keep compiling unchanged.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use proven_shared::{
    AuditEntryId, CausationId, CompanyId, CorrelationId, Page, PageRequest, ProjectId, SessionId,
    TenantId, UserId,
};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::application::ports::{AuditRepository, EventPublisher};
use crate::domain::{
    AuditChange, AuditEntry, AuditExportJob, AuditRetentionPolicy, AuditSearchQuery, CoreError,
};
use crate::events::{ActorRef, CoreEvent, EventEnvelope, ResourceRef};

/// Compute the sha256 hex digest of a JSON payload for immutable audit integrity.
pub fn digest_payload(payload: &serde_json::Value) -> Result<String, CoreError> {
    let bytes = serde_json::to_vec(payload)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
}

/// `integrity_hash = sha256(prev_hash + payload_digest + action + id)` (ADR-0008 §3, optional
/// hash-chain tamper-evidence tier — AUDIT_LOGGING_ARCHITECTURE.md §7).
fn compute_integrity_hash(prev: Option<&str>, digest: &str, action: &str, id: Uuid) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prev.unwrap_or_default().as_bytes());
    hasher.update(digest.as_bytes());
    hasher.update(action.as_bytes());
    hasher.update(id.as_bytes());
    hex::encode(hasher.finalize())
}

/// One `AuditApi::append` / `AuditEngine::record` call. Only `tenant_id`, `actor_type`, `action`,
/// `resource_type`, and `payload` are meaningfully required — every capture field below is
/// optional with a `Default` so existing call sites keep compiling by appending
/// `..Default::default()` (ADR-0008 consequence).
#[derive(Default)]
pub struct AppendAuditEntryCommand {
    pub tenant_id: TenantId,
    pub actor_user_id: Option<UserId>,
    pub actor_type: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub correlation_id: Option<CorrelationId>,
    pub causation_id: Option<CausationId>,
    pub payload: serde_json::Value,

    // --- Audit Engine capture fields (ADR-0008 §1, AUDIT_LOGGING_ARCHITECTURE.md §4) ---
    pub module_key: Option<String>,
    /// Defaults to `"data"` (matches the SQL column default) when omitted.
    pub category: Option<String>,
    /// Defaults to `"success"` when omitted.
    pub outcome: Option<String>,
    pub project_id: Option<ProjectId>,
    pub company_id: Option<CompanyId>,
    pub session_id: Option<SessionId>,
    pub ip_address: Option<String>,
    pub device_id: Option<String>,
    pub user_agent: Option<String>,
    pub workflow_instance_id: Option<Uuid>,
    pub signature_package_id: Option<Uuid>,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub changes: Vec<AuditChange>,
    /// Defaults to `"standard"` when omitted.
    pub retention_class: Option<String>,
    /// Defaults to `"standard"` when omitted.
    pub sensitivity: Option<String>,
}

/// Application service implementing the Audit Engine over an [`AuditRepository`]. Optionally
/// wired to an [`EventPublisher`] so `record`/`request_export` can emit integration events —
/// callers that only need a one-off `append` (most existing services) may omit it via
/// [`AuditService::new`] and simply skip event emission for that call.
pub struct AuditService {
    audit: Arc<dyn AuditRepository>,
    outbox: Option<Arc<dyn EventPublisher>>,
}

/// Primary name going forward — `AuditService` is kept for the original CORE_DOMAIN.md §18 name.
pub type AuditEngine = AuditService;

impl AuditService {
    pub fn new(audit: Arc<dyn AuditRepository>) -> Self {
        Self {
            audit,
            outbox: None,
        }
    }

    /// Attach an outbox so `record`/`request_export` emit `AuditEntryAppended` /
    /// `AuditExportRequested` / `AuditExportCompleted` (ADR-0008 §5).
    #[must_use]
    pub fn with_outbox(mut self, outbox: Arc<dyn EventPublisher>) -> Self {
        self.outbox = Some(outbox);
        self
    }

    /// Append one audit fact. Computes the payload digest and, if a prior entry exists for this
    /// tenant, chains `integrity_prev_hash` → `integrity_hash` (ADR-0008 §3). Never mutates an
    /// existing row — this is the only write path on this service.
    pub async fn append(&self, cmd: AppendAuditEntryCommand) -> Result<AuditEntry, CoreError> {
        let now = Utc::now();
        let payload_digest = digest_payload(&cmd.payload)?;
        let id = AuditEntryId::new();

        let prev_hash = self.audit.last_integrity_hash(cmd.tenant_id).await?;
        let integrity_hash =
            compute_integrity_hash(prev_hash.as_deref(), &payload_digest, &cmd.action, id.as_uuid());

        let changes = serde_json::to_value(&cmd.changes).unwrap_or_else(|_| serde_json::json!([]));

        let entry = AuditEntry {
            id,
            tenant_id: cmd.tenant_id,
            occurred_at: now,
            recorded_at: now,
            actor_user_id: cmd.actor_user_id,
            actor_type: cmd.actor_type,
            action: cmd.action,
            resource_type: cmd.resource_type,
            resource_id: cmd.resource_id,
            correlation_id: cmd.correlation_id,
            causation_id: cmd.causation_id,
            payload: cmd.payload,
            payload_digest,
            module_key: cmd.module_key,
            category: cmd.category.unwrap_or_else(|| "data".to_string()),
            outcome: cmd.outcome.unwrap_or_else(|| "success".to_string()),
            project_id: cmd.project_id,
            company_id: cmd.company_id,
            session_id: cmd.session_id,
            ip_address: cmd.ip_address,
            device_id: cmd.device_id,
            user_agent: cmd.user_agent,
            workflow_instance_id: cmd.workflow_instance_id,
            signature_package_id: cmd.signature_package_id,
            old_value: cmd.old_value,
            new_value: cmd.new_value,
            changes,
            retention_class: cmd.retention_class.unwrap_or_else(|| "standard".to_string()),
            sensitivity: cmd.sensitivity.unwrap_or_else(|| "standard".to_string()),
            integrity_prev_hash: prev_hash,
            integrity_hash: Some(integrity_hash),
        };
        self.audit.append(&entry).await?;

        if let Some(outbox) = &self.outbox {
            outbox
                .publish(EventEnvelope::new(
                    entry.tenant_id,
                    entry
                        .actor_user_id
                        .map(|user_id| ActorRef::User { user_id })
                        .unwrap_or(ActorRef::System),
                    ResourceRef {
                        resource_type: entry.resource_type.clone(),
                        resource_id: entry.resource_id.unwrap_or_else(|| entry.id.as_uuid()),
                    },
                    entry.correlation_id,
                    entry.causation_id,
                    CoreEvent::AuditEntryAppended {
                        tenant_id: entry.tenant_id,
                        audit_entry_id: entry.id,
                    },
                ))
                .await?;
        }

        Ok(entry)
    }

    /// Alias of [`Self::append`] — the primary `AuditEngine` entry point (ADR-0008 §3).
    pub async fn record(&self, cmd: AppendAuditEntryCommand) -> Result<AuditEntry, CoreError> {
        self.append(cmd).await
    }

    /// Back-compat unfiltered paged listing (pre-ADR-0008 `AuditApi::query`).
    pub async fn query(
        &self,
        tenant_id: TenantId,
        page: PageRequest,
    ) -> Result<Page<AuditEntry>, CoreError> {
        self.audit.query(tenant_id, page).await
    }

    /// Filtered, paged audit search (AUDIT_LOGGING_ARCHITECTURE.md §11.2).
    pub async fn search(
        &self,
        tenant_id: TenantId,
        query: AuditSearchQuery,
        page: PageRequest,
    ) -> Result<Page<AuditEntry>, CoreError> {
        self.audit.search(tenant_id, &query, page).await
    }

    /// Page through every entry matching `query` for `tenant_id` — used internally by export and
    /// retention scans. Callers needing an interactive result set should use [`Self::search`]
    /// instead; this method may issue several repository round-trips for large tenants.
    async fn collect_all(
        &self,
        tenant_id: TenantId,
        query: &AuditSearchQuery,
    ) -> Result<Vec<AuditEntry>, CoreError> {
        let mut all = Vec::new();
        let mut offset = 0u32;
        const PAGE_SIZE: u32 = 500;
        loop {
            let page = self
                .audit
                .search(
                    tenant_id,
                    query,
                    PageRequest {
                        limit: PAGE_SIZE,
                        offset,
                    },
                )
                .await?;
            let got = page.items.len() as u32;
            all.extend(page.items);
            if got < PAGE_SIZE || (all.len() as u64) >= page.total {
                break;
            }
            offset += PAGE_SIZE;
        }
        Ok(all)
    }

    /// Request an audit export (ADR-0008 §3, AUDIT_LOGGING_ARCHITECTURE.md §10). Runs
    /// synchronously today — object storage wiring (R2) and an async Temporal workflow are a
    /// follow-up (ADR-0008 consequence); the job still transitions `queued` → `completed` so
    /// callers can integrate against the final API shape now.
    pub async fn request_export(
        &self,
        tenant_id: TenantId,
        requested_by: Option<UserId>,
        filter: AuditSearchQuery,
    ) -> Result<AuditExportJob, CoreError> {
        let now = Utc::now();
        let job_id = Uuid::new_v4();
        let filter_json = serde_json::to_value(&filter)?;

        let mut job = AuditExportJob {
            id: job_id,
            tenant_id,
            requested_by,
            status: "queued".to_string(),
            filter: filter_json,
            entry_count: None,
            storage_key: None,
            error_message: None,
            created_at: now,
            completed_at: None,
        };
        self.audit.insert_export_job(&job).await?;

        if let Some(outbox) = &self.outbox {
            outbox
                .publish(EventEnvelope::new(
                    tenant_id,
                    requested_by
                        .map(|user_id| ActorRef::User { user_id })
                        .unwrap_or(ActorRef::System),
                    ResourceRef {
                        resource_type: "audit_export_job".to_string(),
                        resource_id: job_id,
                    },
                    None,
                    None,
                    CoreEvent::AuditExportRequested { job_id, tenant_id },
                ))
                .await?;
        }

        let entries = self.collect_all(tenant_id, &filter).await?;
        let entry_count = entries.len() as i32;
        let storage_key = format!("audit-exports/{tenant_id}/{job_id}.json");

        job.status = "completed".to_string();
        job.entry_count = Some(entry_count);
        job.storage_key = Some(storage_key.clone());
        job.completed_at = Some(Utc::now());
        self.audit.update_export_job(&job).await?;

        if let Some(outbox) = &self.outbox {
            outbox
                .publish(EventEnvelope::new(
                    tenant_id,
                    ActorRef::System,
                    ResourceRef {
                        resource_type: "audit_export_job".to_string(),
                        resource_id: job_id,
                    },
                    None,
                    None,
                    CoreEvent::AuditExportCompleted {
                        job_id,
                        tenant_id,
                        entry_count,
                        storage_key,
                    },
                ))
                .await?;
        }

        Ok(job)
    }

    pub async fn get_export(
        &self,
        tenant_id: TenantId,
        job_id: Uuid,
    ) -> Result<AuditExportJob, CoreError> {
        self.audit
            .get_export_job(tenant_id, job_id)
            .await?
            .ok_or(CoreError::NotFound("audit_export_job"))
    }

    pub async fn get_retention_policy(
        &self,
        tenant_id: TenantId,
    ) -> Result<AuditRetentionPolicy, CoreError> {
        match self.audit.get_retention_policy(tenant_id).await? {
            Some(policy) => Ok(policy),
            None => Ok(AuditRetentionPolicy::default_for(tenant_id)),
        }
    }

    pub async fn upsert_retention_policy(
        &self,
        mut policy: AuditRetentionPolicy,
    ) -> Result<AuditRetentionPolicy, CoreError> {
        policy.updated_at = Utc::now();
        self.audit.upsert_retention_policy(&policy).await?;
        Ok(policy)
    }

    /// Ids of entries whose age exceeds their [`AuditEntry::retention_class`] threshold under the
    /// tenant's [`AuditRetentionPolicy`] (or the default policy if none is set).
    ///
    /// **Does not delete anything.** Audit facts are append-only (ADR-0008 hard rule); the
    /// returned ids are only *eligible* for the ops-driven archival/export-then-partition-drop
    /// process described in AUDIT_LOGGING_ARCHITECTURE.md §9. Actually removing rows (or dropping
    /// partitions) is a separate, out-of-band operational job outside this engine's scope.
    pub async fn list_purge_candidates(
        &self,
        tenant_id: TenantId,
        now: DateTime<Utc>,
    ) -> Result<Vec<AuditEntryId>, CoreError> {
        let policy = self.get_retention_policy(tenant_id).await?;
        let entries = self.collect_all(tenant_id, &AuditSearchQuery::default()).await?;

        let mut ids = Vec::new();
        for entry in entries {
            let threshold_days = match entry.retention_class.as_str() {
                "security" => policy.security_days,
                "compliance" => policy.compliance_days,
                "restricted" => policy.restricted_days,
                _ => policy.standard_days,
            };
            let age_days = now.signed_duration_since(entry.occurred_at).num_days();
            if age_days >= i64::from(threshold_days) {
                ids.push(entry.id);
            }
        }
        Ok(ids)
    }
}
