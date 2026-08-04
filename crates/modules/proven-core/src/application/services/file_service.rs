//! File management service — upload intents, versions, scan hook, links, temporaries (ADR-0010).
//! Bytes live in R2 (via [`ObjectStoragePort`]); Core owns identity + AuthZ metadata only.

use std::sync::Arc;

use chrono::{Duration, Utc};
use proven_shared::{FileObjectId, TenantId, UserId};
use uuid::Uuid;

use crate::application::ports::{
    AuditRepository, EventPublisher, FileObjectRepository, FileShareLinkRepository,
    ObjectStoragePort, VirusScanHook,
};
use crate::application::services::audit_service::{AppendAuditEntryCommand, AuditService};
use crate::domain::{
    CoreError, DownloadLink, FileLinkKind, FileObject, FileObjectClass, FileObjectStatus,
    FileShareLink, UploadIntent, VirusScanOutcome, VirusScanRequest, VirusScanStatus,
};
use crate::events::{ActorRef, CoreEvent, EventEnvelope, ResourceRef};

const DEFAULT_UPLOAD_TTL_SECS: u64 = 900;
const DEFAULT_PRIVATE_DOWNLOAD_TTL_SECS: u64 = 300;
const DEFAULT_PUBLIC_SHARE_TTL_HOURS: i64 = 72;
const DEFAULT_TEMPORARY_TTL_HOURS: i64 = 24;

pub struct CreateFileUploadIntentCommand {
    pub tenant_id: TenantId,
    pub content_type: Option<String>,
    pub retention_class: Option<String>,
    pub access_class: Option<String>,
    pub created_by: Option<UserId>,
    pub object_class: Option<FileObjectClass>,
    pub original_filename: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub parent_file_id: Option<FileObjectId>,
    pub is_temporary: bool,
    /// Override temporary / share expiry; defaults apply when unset.
    pub expires_at: Option<chrono::DateTime<Utc>>,
}

pub struct CreatePublicShareLinkCommand {
    pub tenant_id: TenantId,
    pub file_id: FileObjectId,
    pub created_by: Option<UserId>,
    pub ttl_hours: Option<i64>,
    pub max_downloads: Option<i32>,
}

pub struct ApplyScanResultCommand {
    pub tenant_id: TenantId,
    pub file_id: FileObjectId,
    pub outcome: VirusScanOutcome,
    pub actor_user_id: Option<UserId>,
}

pub struct FileService {
    files: Arc<dyn FileObjectRepository>,
    links: Arc<dyn FileShareLinkRepository>,
    storage: Arc<dyn ObjectStoragePort>,
    virus_scan: Arc<dyn VirusScanHook>,
    audit: Arc<dyn AuditRepository>,
    outbox: Arc<dyn EventPublisher>,
}

impl FileService {
    pub fn new(
        files: Arc<dyn FileObjectRepository>,
        links: Arc<dyn FileShareLinkRepository>,
        storage: Arc<dyn ObjectStoragePort>,
        virus_scan: Arc<dyn VirusScanHook>,
        audit: Arc<dyn AuditRepository>,
        outbox: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            files,
            links,
            storage,
            virus_scan,
            audit,
            outbox,
        }
    }

    pub async fn create_upload_intent(
        &self,
        cmd: CreateFileUploadIntentCommand,
    ) -> Result<UploadIntent, CoreError> {
        let object_class = cmd.object_class.unwrap_or(FileObjectClass::Attachment);
        let content_type = cmd
            .content_type
            .clone()
            .unwrap_or_else(|| "application/octet-stream".to_string());

        if !object_class.allows_content_type(&content_type) {
            return Err(CoreError::validation(format!(
                "content_type '{content_type}' is not allowed for class {}",
                object_class.as_str()
            )));
        }

        let mut content_version = 1;
        if let Some(parent_id) = cmd.parent_file_id {
            let parent = self
                .files
                .get(cmd.tenant_id, parent_id)
                .await?
                .ok_or(CoreError::NotFound("parent_file_object"))?;
            if parent.status == FileObjectStatus::Deleted {
                return Err(CoreError::conflict("cannot version a deleted file"));
            }
            content_version = parent.content_version + 1;
        }

        let now = Utc::now();
        let id = FileObjectId::new();
        let yyyy = now.format("%Y").to_string();
        let mm = now.format("%m").to_string();
        let filename_safe = sanitize_filename(
            cmd.original_filename
                .as_deref()
                .unwrap_or("original"),
        );
        let storage_key = format!(
            "{}/{}/{}/{}/{}/{}",
            cmd.tenant_id,
            object_class.storage_prefix(),
            yyyy,
            mm,
            id,
            filename_safe
        );

        let expires_at = if cmd.is_temporary {
            Some(
                cmd.expires_at
                    .unwrap_or_else(|| now + Duration::hours(DEFAULT_TEMPORARY_TTL_HOURS)),
            )
        } else {
            cmd.expires_at
        };

        let file = FileObject {
            id,
            tenant_id: cmd.tenant_id,
            status: FileObjectStatus::PendingUpload,
            storage_key: storage_key.clone(),
            content_type: Some(content_type.clone()),
            byte_size: None,
            checksum_sha256: None,
            retention_class: cmd
                .retention_class
                .unwrap_or_else(|| "standard".to_string()),
            access_class: cmd.access_class.unwrap_or_else(|| "tenant".to_string()),
            created_by: cmd.created_by,
            created_at: now,
            updated_at: now,
            version: 1,
            object_class,
            original_filename: cmd.original_filename,
            metadata: cmd.metadata.unwrap_or_else(|| serde_json::json!({})),
            parent_file_id: cmd.parent_file_id,
            content_version,
            is_temporary: cmd.is_temporary,
            expires_at,
            scan_status: VirusScanStatus::NotScanned,
            scan_detail: None,
        };
        self.files.insert(&file).await?;

        let upload = self
            .storage
            .presign_put(&storage_key, &content_type, DEFAULT_UPLOAD_TTL_SECS)
            .await?;

        AuditService::new(self.audit.clone())
            .append(AppendAuditEntryCommand {
                tenant_id: cmd.tenant_id,
                actor_user_id: cmd.created_by,
                actor_type: "user".to_string(),
                action: "core.file.upload_intent_created".to_string(),
                resource_type: "file_object".to_string(),
                resource_id: Some(file.id.as_uuid()),
                correlation_id: None,
                causation_id: None,
                payload: serde_json::json!({
                    "storage_key": file.storage_key,
                    "object_class": object_class.as_str(),
                    "is_temporary": file.is_temporary,
                    "content_version": file.content_version,
                    "parent_file_id": file.parent_file_id,
                }),
                category: Some("data".to_string()),
                ..Default::default()
            })
            .await?;

        self.outbox
            .publish(EventEnvelope::new(
                cmd.tenant_id,
                cmd.created_by
                    .map(|user_id| ActorRef::User { user_id })
                    .unwrap_or(ActorRef::System),
                ResourceRef {
                    resource_type: "file_object".to_string(),
                    resource_id: file.id.as_uuid(),
                },
                None,
                None,
                CoreEvent::FileUploadIntentCreated {
                    tenant_id: cmd.tenant_id,
                    file_id: file.id,
                },
            ))
            .await?;

        Ok(UploadIntent { file, upload })
    }

    pub async fn get_file(
        &self,
        tenant_id: TenantId,
        file_id: FileObjectId,
    ) -> Result<FileObject, CoreError> {
        self.files
            .get(tenant_id, file_id)
            .await?
            .ok_or(CoreError::NotFound("file_object"))
    }

    pub async fn list_versions(
        &self,
        tenant_id: TenantId,
        file_id: FileObjectId,
    ) -> Result<Vec<FileObject>, CoreError> {
        let _ = self.get_file(tenant_id, file_id).await?;
        self.files.list_versions(tenant_id, file_id).await
    }

    pub async fn complete_upload(
        &self,
        tenant_id: TenantId,
        file_id: FileObjectId,
        checksum_sha256: String,
        byte_size: i64,
    ) -> Result<FileObject, CoreError> {
        if checksum_sha256.trim().is_empty() {
            return Err(CoreError::validation("checksum_sha256 must not be empty"));
        }
        if byte_size < 0 {
            return Err(CoreError::validation("byte_size must be non-negative"));
        }

        let mut file = self.get_file(tenant_id, file_id).await?;
        if file.status != FileObjectStatus::PendingUpload {
            return Err(CoreError::conflict("file object is not pending upload"));
        }

        file.checksum_sha256 = Some(checksum_sha256.clone());
        file.byte_size = Some(byte_size);
        file.status = FileObjectStatus::Processing;
        file.scan_status = VirusScanStatus::Pending;
        file.updated_at = Utc::now();
        file.version += 1;
        self.files.update(&file).await?;

        AuditService::new(self.audit.clone())
            .append(AppendAuditEntryCommand {
                tenant_id,
                actor_user_id: file.created_by,
                actor_type: "user".to_string(),
                action: "core.file.upload_completed".to_string(),
                resource_type: "file_object".to_string(),
                resource_id: Some(file.id.as_uuid()),
                correlation_id: None,
                causation_id: None,
                payload: serde_json::json!({
                    "byte_size": byte_size,
                    "checksum_sha256": checksum_sha256,
                }),
                category: Some("data".to_string()),
                ..Default::default()
            })
            .await?;

        let outcome = self
            .virus_scan
            .scan(VirusScanRequest {
                tenant_id,
                file_id: file.id,
                storage_key: file.storage_key.clone(),
                content_type: file.content_type.clone(),
                checksum_sha256,
                byte_size,
                object_class: file.object_class,
            })
            .await?;

        self.apply_scan_outcome(tenant_id, file.id, outcome, file.created_by)
            .await
    }

    pub async fn apply_scan_result(
        &self,
        cmd: ApplyScanResultCommand,
    ) -> Result<FileObject, CoreError> {
        self.apply_scan_outcome(cmd.tenant_id, cmd.file_id, cmd.outcome, cmd.actor_user_id)
            .await
    }

    async fn apply_scan_outcome(
        &self,
        tenant_id: TenantId,
        file_id: FileObjectId,
        outcome: VirusScanOutcome,
        actor_user_id: Option<UserId>,
    ) -> Result<FileObject, CoreError> {
        let mut file = self.get_file(tenant_id, file_id).await?;
        if file.status != FileObjectStatus::Processing {
            return Err(CoreError::conflict(
                "scan result can only be applied while processing",
            ));
        }

        match &outcome {
            VirusScanOutcome::Clean { detail } => {
                file.status = FileObjectStatus::Available;
                file.scan_status = VirusScanStatus::Clean;
                file.scan_detail = detail.clone();
            }
            VirusScanOutcome::Infected { detail } => {
                file.status = FileObjectStatus::Quarantined;
                file.scan_status = VirusScanStatus::Infected;
                file.scan_detail = detail.clone();
            }
            VirusScanOutcome::Pending { detail } => {
                file.status = FileObjectStatus::Processing;
                file.scan_status = VirusScanStatus::Pending;
                file.scan_detail = detail.clone();
            }
            VirusScanOutcome::Error { detail } => {
                file.status = FileObjectStatus::Quarantined;
                file.scan_status = VirusScanStatus::Error;
                file.scan_detail = Some(detail.clone());
            }
        }
        file.updated_at = Utc::now();
        file.version += 1;
        self.files.update(&file).await?;

        let action = match file.scan_status {
            VirusScanStatus::Clean => "core.file.scan_clean",
            VirusScanStatus::Infected => "core.file.scan_infected",
            VirusScanStatus::Pending => "core.file.scan_pending",
            VirusScanStatus::Error => "core.file.scan_error",
            VirusScanStatus::NotScanned => "core.file.scan_unknown",
        };

        AuditService::new(self.audit.clone())
            .append(AppendAuditEntryCommand {
                tenant_id,
                actor_user_id,
                actor_type: if actor_user_id.is_some() {
                    "user".to_string()
                } else {
                    "system".to_string()
                },
                action: action.to_string(),
                resource_type: "file_object".to_string(),
                resource_id: Some(file.id.as_uuid()),
                correlation_id: None,
                causation_id: None,
                payload: serde_json::json!({
                    "status": format!("{:?}", file.status).to_ascii_lowercase(),
                    "scan_status": file.scan_status.as_str(),
                    "scan_detail": file.scan_detail,
                }),
                category: Some("data".to_string()),
                ..Default::default()
            })
            .await?;

        let event = match file.status {
            FileObjectStatus::Available => CoreEvent::FileObjectAvailable {
                tenant_id,
                file_id: file.id,
            },
            FileObjectStatus::Quarantined => CoreEvent::FileObjectQuarantined {
                tenant_id,
                file_id: file.id,
            },
            _ => CoreEvent::FileObjectScanPending {
                tenant_id,
                file_id: file.id,
            },
        };

        self.outbox
            .publish(EventEnvelope::new(
                tenant_id,
                actor_user_id
                    .map(|user_id| ActorRef::User { user_id })
                    .unwrap_or(ActorRef::System),
                ResourceRef {
                    resource_type: "file_object".to_string(),
                    resource_id: file.id.as_uuid(),
                },
                None,
                None,
                event,
            ))
            .await?;

        Ok(file)
    }

    pub async fn soft_delete(
        &self,
        tenant_id: TenantId,
        file_id: FileObjectId,
        deleted_by: Option<UserId>,
    ) -> Result<FileObject, CoreError> {
        let mut file = self.get_file(tenant_id, file_id).await?;
        if file.status == FileObjectStatus::Deleted {
            return Err(CoreError::conflict("file is already deleted"));
        }
        file.status = FileObjectStatus::Deleted;
        file.updated_at = Utc::now();
        file.version += 1;
        self.files.update(&file).await?;

        AuditService::new(self.audit.clone())
            .append(AppendAuditEntryCommand {
                tenant_id,
                actor_user_id: deleted_by,
                actor_type: "user".to_string(),
                action: "core.file.deleted".to_string(),
                resource_type: "file_object".to_string(),
                resource_id: Some(file.id.as_uuid()),
                correlation_id: None,
                causation_id: None,
                payload: serde_json::json!({ "storage_key": file.storage_key }),
                category: Some("data".to_string()),
                ..Default::default()
            })
            .await?;

        self.outbox
            .publish(EventEnvelope::new(
                tenant_id,
                deleted_by
                    .map(|user_id| ActorRef::User { user_id })
                    .unwrap_or(ActorRef::System),
                ResourceRef {
                    resource_type: "file_object".to_string(),
                    resource_id: file.id.as_uuid(),
                },
                None,
                None,
                CoreEvent::FileObjectDeleted {
                    tenant_id,
                    file_id: file.id,
                },
            ))
            .await?;

        Ok(file)
    }

    pub async fn create_private_download_link(
        &self,
        tenant_id: TenantId,
        file_id: FileObjectId,
        requested_by: Option<UserId>,
    ) -> Result<DownloadLink, CoreError> {
        let file = self.get_file(tenant_id, file_id).await?;
        if file.status != FileObjectStatus::Available {
            return Err(CoreError::conflict(
                "only available files can be downloaded",
            ));
        }

        let download = self
            .storage
            .presign_get(
                &file.storage_key,
                DEFAULT_PRIVATE_DOWNLOAD_TTL_SECS,
                file.original_filename.as_deref(),
            )
            .await?;

        AuditService::new(self.audit.clone())
            .append(AppendAuditEntryCommand {
                tenant_id,
                actor_user_id: requested_by,
                actor_type: "user".to_string(),
                action: "core.file.private_link_issued".to_string(),
                resource_type: "file_object".to_string(),
                resource_id: Some(file.id.as_uuid()),
                correlation_id: None,
                causation_id: None,
                payload: serde_json::json!({
                    "expires_at": download.expires_at,
                    "placeholder": download.placeholder,
                }),
                category: Some("data".to_string()),
                ..Default::default()
            })
            .await?;

        Ok(DownloadLink {
            file_id: file.id,
            download,
            link_kind: FileLinkKind::Private,
        })
    }

    pub async fn create_public_share_link(
        &self,
        cmd: CreatePublicShareLinkCommand,
    ) -> Result<FileShareLink, CoreError> {
        let file = self.get_file(cmd.tenant_id, cmd.file_id).await?;
        if file.status != FileObjectStatus::Available {
            return Err(CoreError::conflict(
                "only available files can be shared publicly via API token",
            ));
        }

        let now = Utc::now();
        let ttl = cmd.ttl_hours.unwrap_or(DEFAULT_PUBLIC_SHARE_TTL_HOURS);
        let link = FileShareLink {
            id: Uuid::new_v4(),
            tenant_id: cmd.tenant_id,
            file_id: cmd.file_id,
            token: Uuid::new_v4().as_simple().to_string(),
            kind: FileLinkKind::PublicShare,
            expires_at: now + Duration::hours(ttl),
            created_by: cmd.created_by,
            created_at: now,
            revoked_at: None,
            max_downloads: cmd.max_downloads,
            download_count: 0,
        };
        self.links.insert(&link).await?;

        AuditService::new(self.audit.clone())
            .append(AppendAuditEntryCommand {
                tenant_id: cmd.tenant_id,
                actor_user_id: cmd.created_by,
                actor_type: "user".to_string(),
                action: "core.file.public_link_created".to_string(),
                resource_type: "file_share_link".to_string(),
                resource_id: Some(link.id),
                correlation_id: None,
                causation_id: None,
                payload: serde_json::json!({
                    "file_id": link.file_id,
                    "expires_at": link.expires_at,
                    "max_downloads": link.max_downloads,
                }),
                category: Some("data".to_string()),
                ..Default::default()
            })
            .await?;

        Ok(link)
    }

    pub async fn resolve_public_share_link(
        &self,
        token: &str,
    ) -> Result<DownloadLink, CoreError> {
        let mut link = self
            .links
            .get_by_token(token)
            .await?
            .ok_or(CoreError::NotFound("file_share_link"))?;

        let now = Utc::now();
        if !link.is_usable(now) {
            return Err(CoreError::Forbidden("share link is expired or revoked".into()));
        }

        let file = self.get_file(link.tenant_id, link.file_id).await?;
        if file.status != FileObjectStatus::Available {
            return Err(CoreError::conflict("shared file is not available"));
        }

        let download = self
            .storage
            .presign_get(
                &file.storage_key,
                DEFAULT_PRIVATE_DOWNLOAD_TTL_SECS,
                file.original_filename.as_deref(),
            )
            .await?;

        link.download_count += 1;
        self.links.update(&link).await?;

        AuditService::new(self.audit.clone())
            .append(AppendAuditEntryCommand {
                tenant_id: link.tenant_id,
                actor_user_id: None,
                actor_type: "system".to_string(),
                action: "core.file.public_link_resolved".to_string(),
                resource_type: "file_share_link".to_string(),
                resource_id: Some(link.id),
                correlation_id: None,
                causation_id: None,
                payload: serde_json::json!({
                    "file_id": link.file_id,
                    "download_count": link.download_count,
                }),
                category: Some("data".to_string()),
                ..Default::default()
            })
            .await?;

        Ok(DownloadLink {
            file_id: file.id,
            download,
            link_kind: FileLinkKind::PublicShare,
        })
    }

    pub async fn list_expired_temporaries(
        &self,
        tenant_id: TenantId,
        now: chrono::DateTime<Utc>,
    ) -> Result<Vec<FileObject>, CoreError> {
        self.files.list_expired_temporaries(tenant_id, now).await
    }

    pub async fn update_metadata(
        &self,
        tenant_id: TenantId,
        file_id: FileObjectId,
        metadata: serde_json::Value,
        updated_by: Option<UserId>,
    ) -> Result<FileObject, CoreError> {
        let mut file = self.get_file(tenant_id, file_id).await?;
        if file.status == FileObjectStatus::Deleted {
            return Err(CoreError::conflict("cannot update metadata on deleted file"));
        }
        file.metadata = metadata;
        file.updated_at = Utc::now();
        file.version += 1;
        self.files.update(&file).await?;

        AuditService::new(self.audit.clone())
            .append(AppendAuditEntryCommand {
                tenant_id,
                actor_user_id: updated_by,
                actor_type: "user".to_string(),
                action: "core.file.metadata_updated".to_string(),
                resource_type: "file_object".to_string(),
                resource_id: Some(file.id.as_uuid()),
                correlation_id: None,
                causation_id: None,
                payload: serde_json::json!({ "metadata": file.metadata }),
                category: Some("data".to_string()),
                ..Default::default()
            })
            .await?;

        Ok(file)
    }
}

fn sanitize_filename(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return "original".to_string();
    }
    let safe: String = trimmed
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .take(128)
        .collect();
    if safe.is_empty() {
        "original".to_string()
    } else {
        safe
    }
}
