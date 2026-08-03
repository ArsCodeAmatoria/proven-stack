# Proven — Temporal Workflow Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Temporal Workflow Architecture & Catalog |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Workflow / Platform Architecture |
| **Audience** | Backend, SRE, Domain Engineering |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [System Architecture](./SYSTEM_ARCHITECTURE.md), [Event Catalog](./EVENT_CATALOG.md), [Rust Backend](./RUST_BACKEND_ARCHITECTURE.md), [Go Workers](./GO_WORKERS_ARCHITECTURE.md), module domain docs |

---

## 1. Purpose

This document defines **every Temporal workflow** used by Proven: purpose, inputs, activities, timers, signals, retries, compensation, escalations, and outputs.

**Hard rules**

1. Workflows **orchestrate**; domain modules **decide** invariants via public APIs/activities.  
2. Never bypass Temporal for multi-step compliance processes that need durable timers.  
3. Prefer **Rust activities** for domain commands; **Go activities** for PDF/OCR/media/export I/O.  
4. Activities are **idempotent** (keyed by workflow id + business mutation id).

**Documentation only — no implementation.**

---

## 2. Platform Standards

### 2.1 Task Queues

| Queue | Hosts |
| --- | --- |
| `proven-domain` | Rust activities (module API commands/queries) |
| `proven-io` | Go activities (PDF, OCR, image, heavy export) |
| `proven-notify` | Optional notify-related activities (or fold into domain + NATS) |

### 2.2 Workflow Id Conventions

```text
{tenant_id}:{workflow_type}:{subject_type}:{subject_id}[:{suffix}]
```

Ensures idempotent starts (same id → reject/return existing).

### 2.3 Default Retry (Activities)

| Class | Policy (illustrative) |
| --- | --- |
| Domain command (Rust) | 3–5 attempts; exponential; non-retry on 4xx domain rejects |
| I/O (Go) | More attempts; honor rate limits; heartbeat for long PDF/OCR |
| Notify fan-out | Prefer Notifications module + NATS; activity only when workflow must await |

### 2.4 Compensation Pattern

- Prefer **compensating domain commands** (`Void`, `Cancel`, `Revoke`) over silent undo.  
- Saga steps record undo commands in workflow state.  
- Sealed evidence is never deleted—compensate with void + reason.

### 2.5 Escalation Pattern

Timers fire → activity notifies via Notifications API / events → optionally widen audience or reassign via domain API → emit `EscalationTriggered`.

### 2.6 Visibility

Each start also records `workflows.workflow_instances` via domain API for My Actions / admin visibility.

### 2.7 Signals (Common)

| Signal | Meaning |
| --- | --- |
| `cancel` | User/admin cancel |
| `completed` | External completion hint (still verify via query) |
| `approved` / `rejected` | Human decision |
| `signed` | Signature package progressed |
| `updated` | Subject changed; re-query |

---

## 3. Workflow Catalog

---

### 3.1 DocumentApprovalWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | Drive document version from review through approval to publish (and optional ack fan-out). |
| **Inputs** | `tenant_id`, `document_id`, `document_version_id`, `approval_policy` (steps, due), `publish_options` (`effective_from`?), `start_ack_campaign?` |
| **Activities** | `GetDocumentVersion`; `AssignReviewers`; `NotifyReviewers`; `RecordReviewDecision`; `AssignApprovers`; `RecordApprovalDecision`; `PublishDocumentVersion`; `StartAcknowledgementCampaign` (optional); `AppendAudit` |
| **Timers** | Per-reviewer due; per-approver due; optional delayed `effective_from` publish wait |
| **Signals** | `review_submitted`, `approved`, `rejected`, `cancel`, `content_updated` |
| **Retries** | Domain activities: standard; notify: best-effort with retry |
| **Compensation** | On reject: `ReturnVersionToDraft`; on cancel mid-flight: cancel open assignments; never unpublish silently—use `Withdraw` if needed |
| **Escalations** | Overdue reviewer/approver → notify manager; final escalate to document control admin |
| **Outputs** | `published_version_id` or `rejected` + reasons; optional `assignment_campaign_id` |

---

### 3.2 DocumentAcknowledgementCampaignWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | Ensure required audience acknowledges/signs an effective document version. |
| **Inputs** | `document_version_id`, `assignment_id`, `audience_ref`, `due_at`, `require_signature` |
| **Activities** | `ExpandAudience`; `CreateAckRequests`; `RequestSignaturePackages` (if required); `NotifyAssignees`; `GetAckCompletionStats`; `MarkOverdue` |
| **Timers** | Reminder cadence (T+1d, T+3d, …); due_at; campaign close |
| **Signals** | `ack_completed` (batch), `cancel_campaign`, `version_superseded` |
| **Retries** | Notify retries; domain mark overdue idempotent |
| **Compensation** | On supersede: `CancelPendingAcks` for old version; start new campaign externally |
| **Escalations** | Overdue → supervisor → safety/admin |
| **Outputs** | Completion %; overdue person ids; campaign status |

---

### 3.3 FLHAReviewWorkflow (`SafetyActivityReviewWorkflow` specialized)

| Aspect | Design |
| --- | --- |
| **Purpose** | After FLHA submit: ensure signatures, review, and close—or escalate. |
| **Inputs** | `activity_id`, `project_id`, `require_signatures`, `reviewer_policy`, `sla` |
| **Activities** | `GetActivity`; `CreateSignaturePackage` (if needed); `WaitSignatures` (query/signal); `AssignReviewer`; `NotifyReviewer`; `RecordReview`; `CloseActivity`; `NotifyComplete` |
| **Timers** | Signature due; review SLA; reminder intervals |
| **Signals** | `signed`, `reviewed`, `rejected`, `voided`, `cancel` |
| **Retries** | Signature create/close domain calls idempotent |
| **Compensation** | On void: `VoidActivity`; cancel open signature package |
| **Escalations** | Missing signatures → crew supervisor; review overdue → safety coordinator |
| **Outputs** | `activity_status=closed\|voided`; `signature_package_id?` |

Same pattern applies to generic **`SafetyActivityReviewWorkflow`** for toolbox/inspections with type-specific policy.

---

### 3.4 ToolboxAcknowledgementWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | Multi-signer crew acknowledgement until package sealed. |
| **Inputs** | `activity_id`, `signature_package_id`, `crew_person_ids[]`, `due_at` |
| **Activities** | `NotifyPendingSigners`; `GetPackageStatus`; `RemindSigners`; `OnCompleteMarkActivitySealed` |
| **Timers** | Reminder schedule; due_at |
| **Signals** | `signed`, `package_completed`, `package_voided` |
| **Retries** | Notify best-effort |
| **Compensation** | Void package if activity voided |
| **Escalations** | Incomplete by due → supervisor |
| **Outputs** | Sealed package / expired incomplete |

---

### 3.5 CorrectiveActionSlaWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | Track CA from open through due/overdue to verification/close. |
| **Inputs** | `corrective_action_id`, `due_at`, `owner_person_id`, `verify_required`, `escalation_policy` |
| **Activities** | `GetCA`; `NotifyOwner`; `MarkOverdue` (domain command → event); `NotifyEscalation`; `VerifyCompletion`; `CloseCA` (if policy auto) |
| **Timers** | Reminders before due; due_at; overdue repeat; verify due |
| **Signals** | `completed`, `verified`, `cancelled`, `reassigned`, `due_changed` |
| **Retries** | MarkOverdue must be idempotent |
| **Compensation** | On cancel: stop timers; notify watchers |
| **Escalations** | Owner → supervisor → project safety → tenant safety (policy steps) |
| **Outputs** | Terminal CA status; escalation count |

---

### 3.6 IncidentInvestigationWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | Orchestrate investigation steps, findings/CAs linkage, and close criteria. |
| **Inputs** | `incident_case_id`, `project_id`, `severity`, `investigation_plan` (steps) |
| **Activities** | `OpenInvestigationSteps`; `AssignInvestigators`; `NotifyTeam`; `RecordStepComplete`; `EnsureLinkedCAs`; `EvaluateCloseReadiness`; `CloseIncident`; `NotifyStakeholders` |
| **Timers** | Step SLAs by severity; leadership notification for critical immediate |
| **Signals** | `step_completed`, `ca_linked`, `close_requested`, `reopened`, `cancel` |
| **Retries** | Domain updates retry; critical notify aggressive |
| **Compensation** | Cannot delete incident—compensate with status corrections / reopen |
| **Escalations** | Overdue steps → safety lead → executive for critical |
| **Outputs** | Closed incident + linked CA ids; report hooks |

---

### 3.7 PermitLifecycleWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | Request → issue → monitor validity → suspend/close. |
| **Inputs** | `permit_case_id`, `project_id`, `valid_until?`, `asset_ids[]?` |
| **Activities** | `GetPermit`; `RequestSignatures`; `IssuePermit`; `CheckLinkedAssetReadiness`; `SuspendPermit`; `ClosePermit`; `Notify` |
| **Timers** | Approval due; permit expiry; periodic readiness recheck |
| **Signals** | `approved`, `signed`, `suspend`, `close`, `asset_not_ready` |
| **Retries** | Readiness query retries transient |
| **Compensation** | Suspend on failure; void signatures if never issued |
| **Escalations** | Unapproved request overdue; expired still “active” → force suspend |
| **Outputs** | Terminal permit status |

---

### 3.8 LiftPlanApprovalWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | Multi-party lift plan approval with equipment readiness gates. |
| **Inputs** | `lift_plan_id`, `asset_id`, `approver_slots[]`, `project_id` |
| **Activities** | `GetLiftPlan`; `GetAssetReadiness`; `BlockIfNotReady`; `CreateSignaturePackage`; `NotifyApprovers`; `ApproveLiftPlan`; `NotifyCrew` |
| **Timers** | Approval SLA; readiness recheck interval |
| **Signals** | `readiness_changed`, `approved`, `rejected`, `cancel` |
| **Retries** | Readiness/API standard |
| **Compensation** | Reject/void plan; cancel package |
| **Escalations** | Pending approvers; blocked readiness → equipment manager |
| **Outputs** | Approved/rejected; readiness snapshot id |

---

### 3.9 SafetyBulletinAckWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | Audience acknowledgement of safety bulletin. |
| **Inputs** | `bulletin_id`, `audience`, `due_at`, `require_signature?` |
| **Activities** | `ExpandAudience`; `CreateAcks`; `Notify`; `GetCompletion`; `CloseBulletin` |
| **Timers** | Reminders; due |
| **Signals** | `ack`, `cancel` |
| **Retries** | Notify |
| **Compensation** | Cancel pending on bulletin close/void |
| **Escalations** | Overdue → supervisors |
| **Outputs** | Ack completion stats |

---

### 3.10 TrainingRenewalWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | From expiring completion through renewal assignment to new completion. |
| **Inputs** | `completion_id`, `person_id`, `course_id`, `renewal_window_start`, `valid_to` |
| **Activities** | `EmitExpiring` / confirm status; `OpenRenewalCase`; `CreateTrainingAssignment`; `NotifyWorkerSupervisor`; `AwaitCompletion`; `RecordRenewalComplete`; `EmitExpired` if missed; `DetectCompetencyGap` |
| **Timers** | Expiring window; reminder cadence; hard expiry; renewal overdue |
| **Signals** | `completion_recorded`, `waiver_granted`, `cancel` |
| **Retries** | Assignment create idempotent |
| **Compensation** | Cancel renewal case if course retired; revoke bad completion via domain |
| **Escalations** | Worker → supervisor → training admin; project notify if gap blocks work |
| **Outputs** | New `completion_id` or gap open |

Companion: **`CompletionExpiryWorkflow`** may be thinner (only emit expiring/expired + gap) and start renewal when policy says auto-renew.

---

### 3.11 TrainingRequirementSyncWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | When membership/trade changes, ensure training assignments exist. |
| **Inputs** | `person_id`, `project_id?`, `trigger` (`membership_granted`\|`trade_assigned`\|…) |
| **Activities** | `MatchRequirements`; `UpsertAssignments`; `NotifyNewAssignments` |
| **Timers** | Optional short debounce timer to coalesce bursts |
| **Signals** | `cancel` |
| **Retries** | Upsert idempotent |
| **Compensation** | Cancel assignments if membership revoked (separate revoke handler/workflow) |
| **Escalations** | None typically |
| **Outputs** | Created assignment ids |

---

### 3.12 OrientationDueWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | Ensure site/company orientation completed for project workers. |
| **Inputs** | `person_id`, `project_id`, `orientation_course_id`, `due_at` |
| **Activities** | `EnsureAssignment`; `Notify`; `CheckCompletion`; `FlagGap` |
| **Timers** | Due; reminders |
| **Signals** | `completed`, `membership_revoked` |
| **Retries** | Standard |
| **Compensation** | Cancel on membership revoke |
| **Escalations** | Supervisor if overdue while still member |
| **Outputs** | Complete or gap |

---

### 3.13 EquipmentMaintenanceWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | Orchestrate planned maintenance order through completion and readiness recompute. |
| **Inputs** | `maintenance_order_id`, `asset_id`, `scheduled_for`, `linked_deficiency_ids[]?` |
| **Activities** | `GetOrder`; `NotifyAssignee`; `OptionalTakeOutOfService`; `AwaitCompletionSignal`; `CompleteOrder`; `ClearLinkedDeficiencies` (domain); `RecomputeReadiness`; `NotifyReturned` |
| **Timers** | Schedule start; overdue completion; post-maint verification due |
| **Signals** | `work_started`, `work_completed`, `cancel`, `defer` |
| **Retries** | Readiness recompute idempotent |
| **Compensation** | Cancel order; if OOS auto-set, return to service only via domain rules |
| **Escalations** | Overdue maint → equipment manager → project PM if asset assigned |
| **Outputs** | Order status; readiness state |

---

### 3.14 PeriodicInspectionDueWorkflow / PreUseValidityWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | Due/overdue periodic inspections; track pre-use validity windows. |
| **Inputs** | `asset_id`, `inspection_kind`, `due_at` / `valid_until` |
| **Activities** | `NotifyDue`; `MarkOverdue`; `RecomputeReadiness`; `NotifyBlocked` |
| **Timers** | Due; overdue repeats; pre-use validity end |
| **Signals** | `inspection_completed`, `asset_retired` |
| **Retries** | Standard |
| **Compensation** | Stop on retire |
| **Escalations** | Overdue periodic → manager; blocked on active project → PM/safety |
| **Outputs** | Readiness impact applied |

---

### 3.15 CertificateExpiryWorkflow (Equipment & Training variants)

| Aspect | Design |
| --- | --- |
| **Purpose** | Watch certification/completion validity; emit expiring/expired; drive readiness/gaps. |
| **Inputs** | `subject_type` (`equipment_cert`\|`training_completion`), `subject_id`, `expires_at`, `warn_offsets[]` |
| **Activities** | `EmitExpiring`; `EmitExpired`; `RecomputeEquipmentReadiness` or `OpenCompetencyGap`; `Notify`; `MaybeStartRenewal` / `UpdateBinderCompleteness` |
| **Timers** | Each warn offset; expiry instant |
| **Signals** | `renewed`, `revoked`, `extended` |
| **Retries** | Emit/mark idempotent |
| **Compensation** | On renew: cancel remaining timers |
| **Escalations** | Expired while asset on project / worker on site → ops channels |
| **Outputs** | Expired flag; child workflow ids |

---

### 3.16 BinderCompletenessWatchWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | Keep tower/self-erect binder completeness aligned with cert/inspection events. |
| **Inputs** | `binder_id`, `asset_id` |
| **Activities** | `RecomputeBinder`; `RecomputeReadiness`; `NotifyIfIncompleteOnAssignedProject` |
| **Timers** | Periodic reconcile (e.g., daily) optional |
| **Signals** | `cert_changed`, `inspection_changed`, `section_updated` |
| **Retries** | Standard |
| **Compensation** | N/A |
| **Escalations** | Incomplete binder on Active project assignment |
| **Outputs** | Completeness snapshot |

---

### 3.17 DeficiencySlaWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | Open equipment deficiency through clear/defer with SLA. |
| **Inputs** | `deficiency_id`, `asset_id`, `severity`, `due_at` |
| **Activities** | `Notify`; `MarkOverdue`; `OnClearRecomputeReadiness` |
| **Timers** | Due; overdue |
| **Signals** | `cleared`, `deferred`, `cancel` |
| **Retries** | Standard |
| **Compensation** | Stop on clear |
| **Escalations** | Critical severity immediate + overdue escalate |
| **Outputs** | Terminal deficiency status |

---

### 3.18 OutOfServiceReleaseWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | Controlled return-to-service after OOS with verification steps. |
| **Inputs** | `asset_id`, `verification_checklist_id?`, `required_inspection_kind?` |
| **Activities** | `EnsureInspections`; `EnsureDeficienciesCleared`; `ReturnToService`; `RecomputeReadiness`; `Notify` |
| **Timers** | Verification due |
| **Signals** | `verification_passed`, `cancel` |
| **Retries** | Standard |
| **Compensation** | Remain OOS if verification fails |
| **Escalations** | Stuck OOS overdue verification |
| **Outputs** | Readiness Ready/Restricted/Blocked |

---

### 3.19 GuestSignatureWorkflow / MagicLinkSignatureWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | Time-boxed guest/magic-link signing through seal or expiry. |
| **Inputs** | `signature_package_id`, `slot_id?`, `link_ttl`, `reminder_policy` |
| **Activities** | `IssueMagicLink` (no secret in workflow logs); `NotifyGuest`; `GetPackageStatus`; `ExpirePackageOrLink`; `OnCompleteGenerateCertificate` (Go PDF optional) |
| **Timers** | Link TTL; package expiry; reminders |
| **Signals** | `redeemed`, `signed`, `voided`, `cancel` |
| **Retries** | Certificate gen I/O retries on `proven-io` |
| **Compensation** | Revoke link; void package if required |
| **Escalations** | Optional host notifier if unsigned near expiry |
| **Outputs** | Completed package + certificate id / expired |

**QR variant:** `QrSignSessionWorkflow` — same shape with QR session TTL instead of/in addition to magic link.

---

### 3.20 SignaturePackageWorkflow (Authenticated Multi-Signer)

| Aspect | Design |
| --- | --- |
| **Purpose** | Generic package completion with sequential/parallel slots and reminders. |
| **Inputs** | `signature_package_id`, `ordering`, `expires_at` |
| **Activities** | `NotifyPendingSlots`; `ValidateDocumentVersion` (if doc subject); `GetStatus`; `CompletePackage`; `GenerateEvidenceCertificate` |
| **Timers** | Reminders; expiry |
| **Signals** | `signed`, `declined`, `void`, `version_superseded` |
| **Retries** | Certificate I/O |
| **Compensation** | Void on supersede per policy |
| **Escalations** | Pending slots overdue → assignee managers |
| **Outputs** | Package terminal status + certificate |

**SequentialSigningWorkflow** may be nested or mode of this workflow unlocking slots in order.

---

### 3.21 COR ExternalPrepWorkflow / AuditEngagementWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | Run internal audit or external preparation: gaps, package, score, report. |
| **Inputs** | `engagement_id`, `framework_id`, `subject`, `type` (`internal`\|`external_prep`) |
| **Activities** | `RefreshReadiness`; `ListGaps`; `AssignGapOwners`; `RequestEvidencePackage`; `AwaitPackage`; `CalculateScorecard`; `GenerateAuditReport` (Go PDF); `FinalizeReport`; `CloseEngagement` / mark prep ready; `NotifySponsors` |
| **Timers** | Gap remediation SLAs; package timeout; engagement milestones |
| **Signals** | `package_ready`, `gaps_updated`, `score_approved`, `close`, `cancel` |
| **Retries** | Package assembly long-running with heartbeats; report I/O |
| **Compensation** | Cancel engagement; retain partial package with failed status |
| **Escalations** | Open critical gaps; package failed → COR admin |
| **Outputs** | `package_id`, `report_id`, `score`, engagement status |

---

### 3.22 EvidencePackageWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | Assemble hashed evidence package from provenance refs. |
| **Inputs** | `package_id`, `mapping_refs[]` / engagement scope |
| **Activities** | `ResolveProvenance`; `FetchEvidenceSlices` (module APIs); `HashAndManifest`; `RenderBundle` (Go); `StoreFileComplete`; `MarkPackageReady` / `Failed` |
| **Timers** | Overall deadline |
| **Signals** | `cancel` |
| **Retries** | Per-item fetch retry; continue-on-optional-miss policy |
| **Compensation** | Mark failed; delete incomplete R2 objects if policy allows |
| **Escalations** | Failure → requester + COR admin |
| **Outputs** | `file_object_id`, manifest hash, status |

---

### 3.23 COR GapEscalationWorkflow / ReadinessRecomputeWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | Overdue gap escalation; batch/event readiness recompute. |
| **Inputs** | `gap_id` or `profile_id` |
| **Activities** | `RecomputeCoverage`; `UpdateScore`; `NotifyOnDrop`; `EscalateGap` |
| **Timers** | Gap due; recompute debounce |
| **Signals** | `evidence_linked`, `cancel` |
| **Retries** | Recompute idempotent |
| **Compensation** | N/A |
| **Escalations** | Gap owner chain |
| **Outputs** | Updated readiness score / gap status |

---

### 3.24 NotificationEscalationWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | Durable escalation steps when notification remains unread/unresolved or subject still open. |
| **Inputs** | `notification_id`, `escalation_policy_id`, `subject_ref?` (prefer subject completion signals) |
| **Activities** | `GetNotification`; `CheckSubjectOpen` (domain query); `EscalateNotify` (widen channel/audience); `RecordEscalationStep` |
| **Timers** | Policy step delays |
| **Signals** | `read`, `subject_resolved`, `cancel` |
| **Retries** | Notify delivery |
| **Compensation** | Cancel remaining steps on resolve |
| **Escalations** | *This workflow is the escalation* |
| **Outputs** | Steps executed; final state |

Note: Channel delivery still via Notifications + Go workers; this workflow owns **timing/widening**.

---

### 3.25 DigestScheduleWorkflow (Temporal Schedule + Workflow)

| Aspect | Design |
| --- | --- |
| **Purpose** | Periodic digest batch creation/send. |
| **Inputs** | `digest_schedule_id`, `tenant_id` |
| **Activities** | `CollectEligibleNotifications`; `CreateDigestBatch`; `EnqueueEmail/Teams` |
| **Timers** | Driven by Temporal Schedule cron |
| **Signals** | `cancel_schedule` |
| **Retries** | Batch send |
| **Compensation** | N/A |
| **Escalations** | Send failure → admin |
| **Outputs** | `digest_batch_id` |

---

### 3.26 WorkflowAssignmentWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | Generic human-task assignment: assign owner, remind, escalate, complete—used when a domain step needs durable human ownership without a specialized workflow. |
| **Inputs** | `assignment_id`, `subject_ref`, `assignee_principal`, `due_at`, `escalation_policy`, `task_type` |
| **Activities** | `CreateOrUpdateAssignmentProjection`; `NotifyAssignee`; `QueryCompletion`; `Reassign`; `CompleteAssignment`; `NotifyRequester` |
| **Timers** | Reminders; due; escalate steps |
| **Signals** | `accepted`, `completed`, `rejected`, `reassign`, `cancel` |
| **Retries** | Notify |
| **Compensation** | Cancel assignment; notify |
| **Escalations** | Per policy (assignee → manager → admin) |
| **Outputs** | Assignment terminal status; assignee history |

Used by Admin builder publishes needing human review, ad-hoc COR tasks, etc.

---

### 3.27 TenantAdminOnboardingWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | Provision tenant bootstrap: company, admin user, defaults, branding, license. |
| **Inputs** | `tenant_draft`, `admin_email`, `license_sku`, `branding?` |
| **Activities** | `ProvisionTenant`; `RegisterCompany`; `InviteAdminUser`; `SeedRoles`; `ActivateLicense`; `ApplyBranding`; `NotifyAdmin` |
| **Timers** | Invite acceptance timeout |
| **Signals** | `admin_activated`, `cancel` |
| **Retries** | Provision steps carefully idempotent |
| **Compensation** | Suspend/close tenant on hard fail after partial; support playbook |
| **Escalations** | Ops if stuck |
| **Outputs** | `tenant_id`, admin user id |

---

### 3.28 BuilderPublishWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | Validate and publish Admin builder draft into owning module. |
| **Inputs** | `draft_id`, `target_module`, `target_type` |
| **Activities** | `ValidateDraft`; `PublishToModule`; `RecordPublication`; `NotifyRequester` |
| **Timers** | Optional approval wait if dual-control |
| **Signals** | `approved`, `rejected`, `cancel` |
| **Retries** | Publish idempotent |
| **Compensation** | Mark publication failed; module-side retire if partial |
| **Escalations** | Failed publish → admin |
| **Outputs** | Published foreign id/version |

---

### 3.29 ApiKeyExpiryWorkflow / IntegrationHealthPollWorkflow / SystemHealthProbeWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | Security/ops hygiene timers. |
| **Inputs** | `api_key_id` / `integration_id` / probe scope |
| **Activities** | `WarnExpiring`; `RevokeKey`; `ProbeConnector`; `WriteHealthSnapshot`; `Notify` |
| **Timers** | Warn offsets; poll interval via Schedule |
| **Signals** | `rotated`, `cancel` |
| **Retries** | Probe transient |
| **Compensation** | N/A |
| **Escalations** | Degraded connector critical |
| **Outputs** | Key status / health snapshot |

---

### 3.30 ExportReportWorkflow / AnalyticsExportWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | Durable export/report artifact generation. |
| **Inputs** | `job_id`, `report_key`, `filters`, `format` |
| **Activities** | `AuthorizeJob`; `FetchDataPages`; `RenderArtifact` (Go); `CompleteUpload`; `CompleteExportJob`; `NotifyRequester` |
| **Timers** | Job deadline |
| **Signals** | `cancel` |
| **Retries** | I/O heavy with heartbeat |
| **Compensation** | Mark job failed; delete orphan files if needed |
| **Escalations** | Failure notify |
| **Outputs** | `file_object_id`, job status |

---

### 3.31 EvidenceCertificateWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | Generate signature evidence certificate PDF after package complete. |
| **Inputs** | `signature_package_id` |
| **Activities** | `LoadPackageSnapshot`; `RenderCertificatePdf` (Go); `StoreFile`; `AttachCertificate` |
| **Timers** | Soft deadline |
| **Signals** | `cancel` |
| **Retries** | PDF I/O |
| **Compensation** | Mark cert failed; package remains completed |
| **Escalations** | Ops if repeated fail |
| **Outputs** | `certificate_id`, `file_object_id` |

---

### 3.32 FileMediaProcessingWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | Post-upload AV scan, image derivatives, OCR optional. |
| **Inputs** | `file_object_id`, `processing_profile` |
| **Activities** | `AvScan`; `ProcessImage`; `OcrOptional`; `MarkAvailableOrQuarantine` |
| **Timers** | Processing SLA |
| **Signals** | `cancel` |
| **Retries** | Go I/O retries |
| **Compensation** | Quarantine on AV fail |
| **Escalations** | Quarantine notify uploader/admin |
| **Outputs** | File status; derivative ids |

---

### 3.33 ProjectOnboardingWorkflow

| Aspect | Design |
| --- | --- |
| **Purpose** | After project create/activate: teams, memberships, template controls, orientation requirements. |
| **Inputs** | `project_id`, `template_id?`, `creator_user_id` |
| **Activities** | `ApplyTemplateArtifacts`; `EnsurePrimeParticipant`; `CreateDefaultTeam`; `GrantCreatorMembership`; `SeedTrainingRequirements`; `NotifyPM` |
| **Timers** | Optional checklist completion due |
| **Signals** | `activated`, `cancel` |
| **Retries** | Idempotent seeds |
| **Compensation** | Soft-cancel seeded assignments if project archived immediately |
| **Escalations** | Incomplete onboarding checklist |
| **Outputs** | Onboarding checklist status |

---

## 4. Workflow Index (Quick Reference)

| Workflow | Primary Module |
| --- | --- |
| DocumentApprovalWorkflow | Documents |
| DocumentAcknowledgementCampaignWorkflow | Documents |
| FLHAReviewWorkflow / SafetyActivityReviewWorkflow | Safety |
| ToolboxAcknowledgementWorkflow | Safety |
| CorrectiveActionSlaWorkflow | Safety |
| IncidentInvestigationWorkflow | Safety |
| PermitLifecycleWorkflow | Safety |
| LiftPlanApprovalWorkflow | Safety |
| SafetyBulletinAckWorkflow | Safety |
| TrainingRenewalWorkflow / CompletionExpiryWorkflow | Training |
| TrainingRequirementSyncWorkflow | Training |
| OrientationDueWorkflow | Training |
| EquipmentMaintenanceWorkflow | Equipment |
| PeriodicInspectionDueWorkflow / PreUseValidityWorkflow | Equipment |
| CertificateExpiryWorkflow | Equipment / Training |
| BinderCompletenessWatchWorkflow | Equipment |
| DeficiencySlaWorkflow | Equipment |
| OutOfServiceReleaseWorkflow | Equipment |
| GuestSignatureWorkflow / QrSignSessionWorkflow | Signatures |
| SignaturePackageWorkflow | Signatures |
| EvidenceCertificateWorkflow | Signatures |
| COR AuditEngagement / ExternalPrep | COR |
| EvidencePackageWorkflow | COR |
| GapEscalation / ReadinessRecompute | COR |
| NotificationEscalationWorkflow | Notifications |
| DigestScheduleWorkflow | Notifications |
| WorkflowAssignmentWorkflow | Workflows / cross-cutting |
| TenantAdminOnboardingWorkflow | Core/Admin |
| BuilderPublishWorkflow | Admin |
| ApiKeyExpiry / IntegrationHealth / SystemHealth | Admin |
| ExportReportWorkflow | Analytics/Reports |
| FileMediaProcessingWorkflow | Core Files |
| ProjectOnboardingWorkflow | Projects |

---

## 5. Mapping to User Examples

| Example | Workflow |
| --- | --- |
| Document Approval | `DocumentApprovalWorkflow` |
| FLHA Review | `FLHAReviewWorkflow` |
| Training Renewal | `TrainingRenewalWorkflow` (+ `CompletionExpiryWorkflow`) |
| Corrective Actions | `CorrectiveActionSlaWorkflow` |
| Incident Investigation | `IncidentInvestigationWorkflow` |
| Equipment Maintenance | `EquipmentMaintenanceWorkflow` |
| Certificate Expiry | `CertificateExpiryWorkflow` |
| Guest Signature | `GuestSignatureWorkflow` |
| COR Audit | `AuditEngagementWorkflow` / `ExternalPrepWorkflow` + `EvidencePackageWorkflow` |
| Notification Escalation | `NotificationEscalationWorkflow` |
| Workflow Assignment | `WorkflowAssignmentWorkflow` |

---

## 6. Testing Guidance (Architectural)

- Deterministic workflow tests with time skipping  
- Activity fakes returning domain DTOs  
- Chaos: activity fail mid-saga → compensation assertions  
- Idempotent start with same workflow id  

---

## 7. Success Criteria

Temporal design succeeds when:

1. Every durable compliance process has an explicit workflow above.  
2. Domain invariants remain in module APIs—not workflow conditionals beyond orchestration.  
3. Timers/escalations are recoverable across deploys.  
4. Compensation never destroys sealed evidence.  
5. I/O-heavy work runs on `proven-io` with heartbeats.  
6. Users can see “where is this?” via workflow instance projections.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Temporal Workflow Architecture | Complete Proven workflow catalog |

---

*End of Temporal Workflow Architecture*
