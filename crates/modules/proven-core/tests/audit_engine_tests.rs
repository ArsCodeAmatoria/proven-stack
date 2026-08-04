//! Audit Engine integration tests (ADR-0008, AUDIT_LOGGING_ARCHITECTURE.md) — exercise the
//! expanded `AuditApi` surface through `CoreModule::in_memory()` only, mirroring how every other
//! module is expected to consume Core.

use chrono::{Duration, Utc};
use proven_core::application::services::{AppendAuditEntryCommand, ProvisionTenantCommand, ProvisionTenantResult};
use proven_core::domain::{AuditChange, AuditRetentionPolicy, AuditSearchQuery};
use proven_core::{AuditApi, CoreModule, TenancyApi};
use proven_shared::{CompanyId, PageRequest, ProjectId};

async fn provision_test_tenant(module: &CoreModule) -> ProvisionTenantResult {
    module
        .services
        .provision_tenant(ProvisionTenantCommand {
            slug: "acme-audit".into(),
            display_name: "Acme Construction".into(),
            region_code: proven_shared::RegionCode::new("CA"),
            owner_company_name: "Acme GC Ltd".into(),
            owner_company_type: proven_core::CompanyType::Prime,
            admin_email: "admin@acme-audit.test".into(),
            admin_display_name: "Acme Admin".into(),
            seats_limit: 25,
        })
        .await
        .expect("provision_tenant should succeed")
}

#[tokio::test]
async fn record_captures_user_action_module_project_company_ip_device_changes() {
    let module = CoreModule::in_memory();
    let result = provision_test_tenant(&module).await;
    let project_id = ProjectId::new();
    let company_id = CompanyId::new();

    let entry = module
        .services
        .append(AppendAuditEntryCommand {
            tenant_id: result.tenant.id,
            actor_user_id: Some(result.admin_user.id),
            actor_type: "user".into(),
            action: "safety.activity.submitted".into(),
            resource_type: "safety_activity".into(),
            resource_id: None,
            payload: serde_json::json!({ "activity": "toolbox_talk" }),
            module_key: Some("safety".into()),
            category: Some("data".into()),
            project_id: Some(project_id),
            company_id: Some(company_id),
            ip_address: Some("203.0.113.7".into()),
            device_id: Some("device-42".into()),
            changes: vec![AuditChange {
                field: "status".into(),
                old: Some(serde_json::json!("draft")),
                new: Some(serde_json::json!("submitted")),
            }],
            ..Default::default()
        })
        .await
        .expect("record should succeed");

    assert_eq!(entry.module_key.as_deref(), Some("safety"));
    assert_eq!(entry.category, "data");
    assert_eq!(entry.outcome, "success", "outcome should default to success");
    assert_eq!(entry.project_id, Some(project_id));
    assert_eq!(entry.company_id, Some(company_id));
    assert_eq!(entry.ip_address.as_deref(), Some("203.0.113.7"));
    assert_eq!(entry.device_id.as_deref(), Some("device-42"));
    assert_eq!(entry.actor_user_id, Some(result.admin_user.id));
    assert_eq!(entry.action, "safety.activity.submitted");

    let changes = entry
        .changes
        .as_array()
        .expect("changes should serialize as a JSON array");
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0]["field"], "status");
    assert_eq!(changes[0]["old"], serde_json::json!("draft"));
    assert_eq!(changes[0]["new"], serde_json::json!("submitted"));
}

#[tokio::test]
async fn search_filters_by_module_and_project() {
    let module = CoreModule::in_memory();
    let result = provision_test_tenant(&module).await;
    let safety_project = ProjectId::new();
    let documents_project = ProjectId::new();

    module
        .services
        .append(AppendAuditEntryCommand {
            tenant_id: result.tenant.id,
            actor_user_id: Some(result.admin_user.id),
            actor_type: "user".into(),
            action: "safety.activity.submitted".into(),
            resource_type: "safety_activity".into(),
            payload: serde_json::json!({}),
            module_key: Some("safety".into()),
            project_id: Some(safety_project),
            ..Default::default()
        })
        .await
        .expect("append should succeed");

    module
        .services
        .append(AppendAuditEntryCommand {
            tenant_id: result.tenant.id,
            actor_user_id: Some(result.admin_user.id),
            actor_type: "user".into(),
            action: "documents.version.published".into(),
            resource_type: "document_version".into(),
            payload: serde_json::json!({}),
            module_key: Some("documents".into()),
            project_id: Some(documents_project),
            ..Default::default()
        })
        .await
        .expect("append should succeed");

    let safety_results = module
        .services
        .search(
            result.tenant.id,
            AuditSearchQuery {
                module_key: Some("safety".to_string()),
                ..Default::default()
            },
            PageRequest::default(),
        )
        .await
        .expect("search should succeed");
    assert_eq!(safety_results.items.len(), 1);
    assert_eq!(safety_results.items[0].module_key.as_deref(), Some("safety"));

    let project_results = module
        .services
        .search(
            result.tenant.id,
            AuditSearchQuery {
                project_id: Some(documents_project),
                ..Default::default()
            },
            PageRequest::default(),
        )
        .await
        .expect("search should succeed");
    assert_eq!(project_results.items.len(), 1);
    assert_eq!(project_results.items[0].action, "documents.version.published");

    let no_match = module
        .services
        .search(
            result.tenant.id,
            AuditSearchQuery {
                module_key: Some("training".to_string()),
                ..Default::default()
            },
            PageRequest::default(),
        )
        .await
        .expect("search should succeed");
    assert!(no_match.items.is_empty());
}

#[tokio::test]
async fn old_new_value_and_changes_round_trip() {
    let module = CoreModule::in_memory();
    let result = provision_test_tenant(&module).await;

    let old_value = serde_json::json!({ "seats_limit": 25 });
    let new_value = serde_json::json!({ "seats_limit": 50 });

    let entry = module
        .services
        .append(AppendAuditEntryCommand {
            tenant_id: result.tenant.id,
            actor_user_id: Some(result.admin_user.id),
            actor_type: "user".into(),
            action: "core.license.updated".into(),
            resource_type: "license".into(),
            payload: serde_json::json!({}),
            old_value: Some(old_value.clone()),
            new_value: Some(new_value.clone()),
            changes: vec![AuditChange {
                field: "seats_limit".into(),
                old: Some(serde_json::json!(25)),
                new: Some(serde_json::json!(50)),
            }],
            ..Default::default()
        })
        .await
        .expect("append should succeed");

    assert_eq!(entry.old_value, Some(old_value.clone()));
    assert_eq!(entry.new_value, Some(new_value.clone()));

    // Round-trip through search — old/new values and changes must survive storage untouched.
    let page = module
        .services
        .search(
            result.tenant.id,
            AuditSearchQuery {
                action: Some("core.license.updated".to_string()),
                ..Default::default()
            },
            PageRequest::default(),
        )
        .await
        .expect("search should succeed");
    let fetched = page.items.first().expect("entry should be found");
    assert_eq!(fetched.old_value, Some(old_value));
    assert_eq!(fetched.new_value, Some(new_value));
    assert_eq!(fetched.changes, entry.changes);
}

#[tokio::test]
async fn export_job_completes_with_entries() {
    let module = CoreModule::in_memory();
    let result = provision_test_tenant(&module).await;

    for i in 0..3 {
        module
            .services
            .append(AppendAuditEntryCommand {
                tenant_id: result.tenant.id,
                actor_user_id: Some(result.admin_user.id),
                actor_type: "user".into(),
                action: format!("test.export.{i}"),
                resource_type: "test_resource".into(),
                payload: serde_json::json!({ "i": i }),
                ..Default::default()
            })
            .await
            .expect("append should succeed");
    }

    let job = module
        .services
        .request_export(
            result.tenant.id,
            Some(result.admin_user.id),
            AuditSearchQuery {
                action: Some("test.export.0".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect("request_export should succeed");

    assert_eq!(job.status, "completed");
    assert_eq!(job.entry_count, Some(1));
    assert!(job
        .storage_key
        .as_deref()
        .expect("storage_key should be set")
        .starts_with(&format!("audit-exports/{}/", result.tenant.id)));
    assert!(job.completed_at.is_some());

    let fetched = module
        .services
        .get_export(result.tenant.id, job.id)
        .await
        .expect("get_export should succeed");
    assert_eq!(fetched.status, "completed");
    assert_eq!(fetched.entry_count, Some(1));

    // Unfiltered export should pick up all appended entries plus the tenant-provisioning entry.
    let full_job = module
        .services
        .request_export(result.tenant.id, Some(result.admin_user.id), AuditSearchQuery::default())
        .await
        .expect("request_export should succeed");
    assert!(full_job.entry_count.unwrap_or(0) >= 4);
}

#[tokio::test]
async fn retention_policy_lists_purge_candidates_without_deleting() {
    let module = CoreModule::in_memory();
    let result = provision_test_tenant(&module).await;

    module
        .services
        .append(AppendAuditEntryCommand {
            tenant_id: result.tenant.id,
            actor_user_id: Some(result.admin_user.id),
            actor_type: "user".into(),
            action: "test.retention.candidate".into(),
            resource_type: "test_resource".into(),
            payload: serde_json::json!({}),
            retention_class: Some("standard".to_string()),
            ..Default::default()
        })
        .await
        .expect("append should succeed");

    // Tight retention window: anything at least 1 day old is eligible for archival.
    module
        .services
        .upsert_retention_policy(AuditRetentionPolicy {
            tenant_id: result.tenant.id,
            standard_days: 1,
            security_days: 2555,
            compliance_days: 2555,
            restricted_days: 3650,
            export_before_purge: true,
            updated_at: Utc::now(),
        })
        .await
        .expect("upsert_retention_policy should succeed");

    let policy = module
        .services
        .get_retention_policy(result.tenant.id)
        .await
        .expect("get_retention_policy should succeed");
    assert_eq!(policy.standard_days, 1);

    let too_soon = module
        .services
        .list_audit_purge_candidates(result.tenant.id, Utc::now())
        .await
        .expect("list_purge_candidates should succeed");
    assert!(
        too_soon.is_empty(),
        "nothing should be eligible immediately after append"
    );

    let count_before = module
        .services
        .search(result.tenant.id, AuditSearchQuery::default(), PageRequest::default())
        .await
        .expect("search should succeed")
        .total;

    let later = Utc::now() + Duration::days(10);
    let candidates = module
        .services
        .list_audit_purge_candidates(result.tenant.id, later)
        .await
        .expect("list_purge_candidates should succeed");
    assert!(
        !candidates.is_empty(),
        "standard-class entries older than the 1-day policy should be eligible"
    );

    // Hard rule: listing candidates must never delete or mutate audit facts.
    let count_after = module
        .services
        .search(result.tenant.id, AuditSearchQuery::default(), PageRequest::default())
        .await
        .expect("search should succeed")
        .total;
    assert_eq!(count_before, count_after);
}

#[tokio::test]
async fn append_only_digest_stable() {
    let module = CoreModule::in_memory();
    let result = provision_test_tenant(&module).await;

    let payload = serde_json::json!({ "hello": "world" });
    let entry1 = module
        .services
        .append(AppendAuditEntryCommand {
            tenant_id: result.tenant.id,
            actor_user_id: Some(result.admin_user.id),
            actor_type: "user".into(),
            action: "test.action".into(),
            resource_type: "test_resource".into(),
            payload: payload.clone(),
            ..Default::default()
        })
        .await
        .expect("append should succeed");

    let expected_digest =
        proven_core::application::services::audit_service::digest_payload(&payload)
            .expect("digest should compute");
    assert_eq!(entry1.payload_digest, expected_digest);
    assert!(
        entry1.integrity_hash.is_some(),
        "every appended entry should get an integrity hash"
    );

    let entry2 = module
        .services
        .append(AppendAuditEntryCommand {
            tenant_id: result.tenant.id,
            actor_user_id: Some(result.admin_user.id),
            actor_type: "user".into(),
            action: "test.action".into(),
            resource_type: "test_resource".into(),
            payload,
            ..Default::default()
        })
        .await
        .expect("append should succeed");

    // Append-only: distinct ids, stable digest, and the hash chain links entry2 to entry1.
    assert_ne!(entry1.id, entry2.id);
    assert_eq!(entry1.payload_digest, entry2.payload_digest);
    assert_eq!(entry2.integrity_prev_hash, entry1.integrity_hash);
    assert_ne!(entry2.integrity_hash, entry1.integrity_hash);
}
