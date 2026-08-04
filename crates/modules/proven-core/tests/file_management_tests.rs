//! File management engine tests (ADR-0010).

use proven_core::application::services::{
    ApplyScanResultCommand, CreateFileUploadIntentCommand, CreatePublicShareLinkCommand,
};
use proven_core::domain::{
    FileObjectClass, FileObjectStatus, VirusScanOutcome, VirusScanStatus,
};
use proven_core::infrastructure::virus_scan::EnqueuePendingVirusScanHook;
use proven_core::{
    CoreModule, CorePorts, CoreServices, FileApi, PlaceholderObjectStorage,
};
use proven_shared::TenantId;
use std::sync::Arc;

#[tokio::test]
async fn upload_scan_download_and_share_flow() {
    let module = CoreModule::in_memory();
    let tenant_id = TenantId::new();

    let intent = module
        .services
        .create_upload_intent(CreateFileUploadIntentCommand {
            tenant_id,
            content_type: Some("image/jpeg".into()),
            retention_class: None,
            access_class: None,
            created_by: None,
            object_class: Some(FileObjectClass::Photo),
            original_filename: Some("site photo.jpg".into()),
            metadata: Some(serde_json::json!({ "source": "mobile" })),
            parent_file_id: None,
            is_temporary: false,
            expires_at: None,
        })
        .await
        .expect("intent");

    assert_eq!(intent.file.status, FileObjectStatus::PendingUpload);
    assert_eq!(intent.file.object_class, FileObjectClass::Photo);
    assert!(intent.upload.placeholder);
    assert!(intent.file.storage_key.contains("/images/"));
    assert!(intent.file.original_filename.as_deref() == Some("site photo.jpg")
        || intent.file.storage_key.contains("site_photo.jpg"));

    let available = module
        .services
        .complete_upload(
            tenant_id,
            intent.file.id,
            "abc123".into(),
            1024,
        )
        .await
        .expect("complete + passthrough scan");

    assert_eq!(available.status, FileObjectStatus::Available);
    assert_eq!(available.scan_status, VirusScanStatus::Clean);

    let private = module
        .services
        .create_private_download_link(tenant_id, available.id, None)
        .await
        .expect("private link");
    assert_eq!(private.download.method, "GET");

    let share = module
        .services
        .create_public_share_link(CreatePublicShareLinkCommand {
            tenant_id,
            file_id: available.id,
            created_by: None,
            ttl_hours: Some(1),
            max_downloads: Some(2),
        })
        .await
        .expect("public share");

    let resolved = module
        .services
        .resolve_public_share_link(&share.token)
        .await
        .expect("resolve share");
    assert_eq!(resolved.file_id, available.id);
}

#[tokio::test]
async fn versioning_and_metadata() {
    let module = CoreModule::in_memory();
    let tenant_id = TenantId::new();

    let v1 = module
        .services
        .create_upload_intent(CreateFileUploadIntentCommand {
            tenant_id,
            content_type: Some("application/pdf".into()),
            retention_class: Some("evidence".into()),
            access_class: None,
            created_by: None,
            object_class: Some(FileObjectClass::Pdf),
            original_filename: Some("drawing.pdf".into()),
            metadata: None,
            parent_file_id: None,
            is_temporary: false,
            expires_at: None,
        })
        .await
        .expect("v1 intent");
    module
        .services
        .complete_upload(tenant_id, v1.file.id, "v1".into(), 10)
        .await
        .expect("v1 complete");

    let v2 = module
        .services
        .create_upload_intent(CreateFileUploadIntentCommand {
            tenant_id,
            content_type: Some("application/pdf".into()),
            retention_class: None,
            access_class: None,
            created_by: None,
            object_class: Some(FileObjectClass::Pdf),
            original_filename: Some("drawing-v2.pdf".into()),
            metadata: None,
            parent_file_id: Some(v1.file.id),
            is_temporary: false,
            expires_at: None,
        })
        .await
        .expect("v2 intent");
    assert_eq!(v2.file.content_version, 2);
    assert_eq!(v2.file.parent_file_id, Some(v1.file.id));

    let updated = module
        .services
        .update_file_metadata(
            tenant_id,
            v1.file.id,
            serde_json::json!({ "sensitivity": "restricted" }),
            None,
        )
        .await
        .expect("metadata");
    assert_eq!(updated.metadata["sensitivity"], "restricted");

    let versions = module
        .services
        .list_file_versions(tenant_id, v1.file.id)
        .await
        .expect("versions");
    assert!(versions.len() >= 2);
}

#[tokio::test]
async fn temporary_upload_expiry_candidates() {
    let module = CoreModule::in_memory();
    let tenant_id = TenantId::new();
    let past = chrono::Utc::now() - chrono::Duration::hours(1);

    let intent = module
        .services
        .create_upload_intent(CreateFileUploadIntentCommand {
            tenant_id,
            content_type: Some("application/octet-stream".into()),
            retention_class: None,
            access_class: None,
            created_by: None,
            object_class: Some(FileObjectClass::Attachment),
            original_filename: None,
            metadata: None,
            parent_file_id: None,
            is_temporary: true,
            expires_at: Some(past),
        })
        .await
        .expect("temp");

    let expired = module
        .services
        .list_expired_temporaries(tenant_id, chrono::Utc::now())
        .await
        .expect("list");
    assert_eq!(expired.len(), 1);
    assert_eq!(expired[0].id, intent.file.id);
}

#[tokio::test]
async fn content_type_gated_by_class() {
    let module = CoreModule::in_memory();
    let err = module
        .services
        .create_upload_intent(CreateFileUploadIntentCommand {
            tenant_id: TenantId::new(),
            content_type: Some("video/mp4".into()),
            retention_class: None,
            access_class: None,
            created_by: None,
            object_class: Some(FileObjectClass::Photo),
            original_filename: None,
            metadata: None,
            parent_file_id: None,
            is_temporary: false,
            expires_at: None,
        })
        .await;
    assert!(err.is_err());
}

#[tokio::test]
async fn enqueue_scan_hook_leaves_processing() {
    let mut ports = CorePorts::in_memory();
    ports.virus_scan = Arc::new(EnqueuePendingVirusScanHook);
    ports.object_storage = Arc::new(PlaceholderObjectStorage::new());
    let services = CoreServices::new(ports);

    let tenant_id = TenantId::new();
    let intent = services
        .create_upload_intent(CreateFileUploadIntentCommand {
            tenant_id,
            content_type: Some("application/pdf".into()),
            retention_class: None,
            access_class: None,
            created_by: None,
            object_class: Some(FileObjectClass::Certificate),
            original_filename: Some("cert.pdf".into()),
            metadata: None,
            parent_file_id: None,
            is_temporary: false,
            expires_at: None,
        })
        .await
        .expect("intent");

    let processing = services
        .complete_upload(tenant_id, intent.file.id, "hash".into(), 99)
        .await
        .expect("complete");
    assert_eq!(processing.status, FileObjectStatus::Processing);
    assert_eq!(processing.scan_status, VirusScanStatus::Pending);

    let clean = services
        .apply_scan_result(ApplyScanResultCommand {
            tenant_id,
            file_id: processing.id,
            outcome: VirusScanOutcome::Clean {
                detail: Some("clamav_ok".into()),
            },
            actor_user_id: None,
        })
        .await
        .expect("worker callback");
    assert_eq!(clean.status, FileObjectStatus::Available);

    let infected_intent = services
        .create_upload_intent(CreateFileUploadIntentCommand {
            tenant_id,
            content_type: Some("application/pdf".into()),
            retention_class: None,
            access_class: None,
            created_by: None,
            object_class: Some(FileObjectClass::Drawing),
            original_filename: None,
            metadata: None,
            parent_file_id: None,
            is_temporary: false,
            expires_at: None,
        })
        .await
        .expect("intent2");
    let pending = services
        .complete_upload(tenant_id, infected_intent.file.id, "x".into(), 1)
        .await
        .expect("complete2");
    let quarantined = services
        .apply_scan_result(ApplyScanResultCommand {
            tenant_id,
            file_id: pending.id,
            outcome: VirusScanOutcome::Infected {
                detail: Some("eicar".into()),
            },
            actor_user_id: None,
        })
        .await
        .expect("quarantine");
    assert_eq!(quarantined.status, FileObjectStatus::Quarantined);

    let blocked = services
        .create_private_download_link(tenant_id, quarantined.id, None)
        .await;
    assert!(blocked.is_err());
}
