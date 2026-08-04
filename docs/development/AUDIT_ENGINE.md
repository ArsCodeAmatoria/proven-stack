# Audit Engine (developer notes)

Canonical design: [ADR-0008: Audit Engine in Core](../adr/0008-audit-engine.md) and
[AUDIT_LOGGING_ARCHITECTURE.md](../architecture/AUDIT_LOGGING_ARCHITECTURE.md).

Core `AuditApi` is the platform's **only** audit Source of Record. Modules append through it —
they never keep a second authoritative audit store (Users' profile audit log is a thin UX
convenience layer, not a compliance SoR).

## Crate

`crates/modules/proven-core`:

| Layer | File |
| --- | --- |
| Domain types | `src/domain/audit.rs` (`AuditChange`, `AuditCategory`, `AuditOutcome`, `AuditRetentionClass`, `AuditSearchQuery`, `AuditExportJob`, `AuditRetentionPolicy`), `src/domain/models.rs` (`AuditEntry`) |
| Engine | `src/application/services/audit_service.rs` (`AuditService` / `AuditEngine` type alias) |
| Port | `src/application/ports.rs` (`AuditRepository`) |
| Adapters | `src/infrastructure/memory.rs` (`MemoryStore`), `src/infrastructure/postgres.rs` (`PgAuditRepository`) |
| Public interface | `src/application/apis.rs` (`AuditApi`) |
| HTTP | `src/api/handlers.rs`, `src/api/router.rs` — `/api/v1/core/audit*` |
| Migration | `db/migrations/core/20260803240000_core_audit_engine.sql` |

## Hard rules

1. **Append-only.** `AuditService` exposes no update/delete method. Corrections are new entries
   referencing the original by `resource_id`/`correlation_id`.
2. **No secrets in payloads.** `payload`, `old_value`, `new_value`, and `changes` must never carry
   passwords, magic-link secrets, TOTP seeds, raw signature strokes, or medical note bodies.
3. **`core.audit_entries` is the only SoR.** Do not add a parallel audit table for a new module —
   extend this engine's capture fields instead.
4. **Retention never deletes.** `AuditService::list_purge_candidates` only returns eligible ids;
   archival/export-then-partition-drop is a separate, ops-driven process.

## Recording an entry

```rust
use proven_core::application::services::AppendAuditEntryCommand;

audit_api
    .append(AppendAuditEntryCommand {
        tenant_id,
        actor_user_id: Some(user_id),
        actor_type: "user".to_string(),
        action: "safety.activity.submitted".to_string(),
        resource_type: "safety_activity".to_string(),
        resource_id: Some(activity_id),
        payload: serde_json::json!({ "activity_id": activity_id }),
        module_key: Some("safety".to_string()),
        category: Some("data".to_string()),
        project_id: Some(project_id),
        ..Default::default()
    })
    .await?;
```

Only `tenant_id`, `actor_type`, `action`, `resource_type`, and `payload` are meaningfully
required — every ADR-0008 capture field (`module_key`, `category`, `outcome`, `project_id`,
`company_id`, `session_id`, `ip_address`, `device_id`, `user_agent`, `workflow_instance_id`,
`signature_package_id`, `old_value`, `new_value`, `changes`, `retention_class`, `sensitivity`) is
optional with a `Default`, so pre-ADR-0008 call sites keep compiling unchanged by appending
`..Default::default()`. Omitted fields default to `category = "data"`, `outcome = "success"`,
`retention_class = "standard"`, `sensitivity = "standard"` — matching the SQL column defaults.

`record` is an alias of `append` — new code should prefer `AuditEngine::record` /
`AuditApi::append` interchangeably; both compute the payload digest and chain
`integrity_prev_hash` → `integrity_hash` from the previous entry for that tenant (optional
tamper-evidence tier, AUDIT_LOGGING_ARCHITECTURE.md §7).

## Searching

`AuditApi::search(tenant_id, AuditSearchQuery, PageRequest)` filters on actor, action, module,
category, project, company, resource, workflow/signature refs, outcome, time range, and a
substring match (`q`) against `action` or the stringified payload. `AuditApi::query` remains as an
unfiltered back-compat wrapper (`AuditSearchQuery::default()`).

HTTP: `GET /api/v1/core/audit` (query params mirror `AuditSearchQuery`), permission
`core.audit.read`.

## Exports

`AuditApi::request_export(tenant_id, requested_by, filter)` creates an `AuditExportJob`
(`queued` → `completed`) and returns it once done. Today the "export" runs synchronously in
process and only records the entry count + a placeholder `storage_key`
(`audit-exports/{tenant_id}/{job_id}.json`) — actual R2 upload and an async Temporal export
workflow are a follow-up (ADR-0008 consequence). `AuditExportRequested` / `AuditExportCompleted`
events are published when an outbox is wired.

HTTP: `POST /api/v1/core/audit/exports`, `GET /api/v1/core/audit/exports/{id}`, permission
`core.audit.export`.

## Retention

`AuditApi::get_retention_policy` / `upsert_retention_policy` manage per-tenant
`AuditRetentionPolicy` (`standard_days`, `security_days`, `compliance_days`, `restricted_days`,
`export_before_purge`). Unset tenants get sensible in-memory defaults
(`AuditRetentionPolicy::default_for`) matching the SQL column defaults — callers never need to
handle a "no policy" error case.

`AuditService::list_purge_candidates(tenant_id, now)` (not part of the public `AuditApi` — an
ops/back-office concern) returns `AuditEntryId`s whose age exceeds their retention class's
threshold. **It never deletes anything.** Archival (export → partition drop) remains a separate
operational job per AUDIT_LOGGING_ARCHITECTURE.md §9.

HTTP: `GET`/`PUT /api/v1/core/audit/retention-policy`, permission `core.audit.read` /
`core.audit.export` respectively.

## Events

`proven.core.v1.*` (published only when `AuditService::with_outbox` is set — `CoreServices` wires
the platform outbox by default):

- `AuditEntryAppended { tenant_id, audit_entry_id }`
- `AuditExportRequested { job_id, tenant_id }`
- `AuditExportCompleted { job_id, tenant_id, entry_count, storage_key }`

## Tests

`crates/modules/proven-core/tests/audit_engine_tests.rs` exercises the engine end to end through
`CoreModule::in_memory()`: capture-field round-trips, module/project search filters, old/new
value + change-diff persistence, export job completion, retention-policy-driven purge candidate
listing (without deletion), and the append-only integrity hash chain.
