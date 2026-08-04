# ADR-0008: Audit Engine in Core

| Field | Value |
| --- | --- |
| Status | Accepted |
| Date | 2026-08-03 |
| Deciders | Lead Software Engineering, Compliance |

## Context

Platform rule: every compliance- or security-significant action must be audited ([AUDIT_LOGGING_ARCHITECTURE.md](../architecture/AUDIT_LOGGING_ARCHITECTURE.md), AGENTS.md). Core already owns `AuditApi` / `core.audit_entries`. Product needs a richer **Audit Engine**: user, action, module, timestamp, project, company, IP, device, change diffs, workflow/signature refs, retention, search, and exports.

## Decision

1. **Extend Core audit** — do not create a second audit SoR (Users’ profile audit remains a thin UX log; security/compliance SoR is Core).
2. Enrich `AuditEntry` with capture fields aligned to architecture §4 (module, project_id, company_id, ip, device, old/new values, workflow/signature refs, category, outcome, retention class).
3. Implement **`AuditEngine`** (application service) over `AuditApi`: `record`, `search`, `request_export`, `apply_retention_policy` (export/archive markers — no in-place delete of facts).
4. **Append-only** — corrections are new rows; retention means cold export + partition drop by ops policy, not UPDATE of audit facts.
5. Emit `AuditEntryAppended` / `AuditExportRequested` / `AuditExportCompleted` events (`proven.core.v1.*`).
6. Search filters: actor, action, module, project, company, time range, resource, workflow, signature, free-text on redacted payload.
7. No secrets in payloads (existing hard rule).

## Consequences

- Existing `AppendAuditEntryCommand` callers remain valid; new fields are optional with defaults.
- Export artifacts are metadata + JSON snapshot in-memory/R2 key placeholder until object storage wiring.
- Full WORM Object Lock is a future ops concern; engine records export jobs today.
