-- Audit Engine enrichment (ADR-0008). Append-only; expand columns only.

ALTER TABLE core.audit_entries
  ADD COLUMN IF NOT EXISTS recorded_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  ADD COLUMN IF NOT EXISTS module_key TEXT,
  ADD COLUMN IF NOT EXISTS category TEXT NOT NULL DEFAULT 'data'
    CHECK (category IN ('auth', 'authz', 'data', 'signature', 'workflow', 'admin', 'export', 'other')),
  ADD COLUMN IF NOT EXISTS outcome TEXT NOT NULL DEFAULT 'success'
    CHECK (outcome IN ('success', 'deny', 'failure')),
  ADD COLUMN IF NOT EXISTS project_id UUID,
  ADD COLUMN IF NOT EXISTS company_id UUID,
  ADD COLUMN IF NOT EXISTS session_id UUID,
  ADD COLUMN IF NOT EXISTS ip_address TEXT,
  ADD COLUMN IF NOT EXISTS device_id TEXT,
  ADD COLUMN IF NOT EXISTS user_agent TEXT,
  ADD COLUMN IF NOT EXISTS workflow_instance_id UUID,
  ADD COLUMN IF NOT EXISTS signature_package_id UUID,
  ADD COLUMN IF NOT EXISTS old_value JSONB,
  ADD COLUMN IF NOT EXISTS new_value JSONB,
  ADD COLUMN IF NOT EXISTS changes JSONB NOT NULL DEFAULT '[]'::jsonb,
  ADD COLUMN IF NOT EXISTS retention_class TEXT NOT NULL DEFAULT 'standard'
    CHECK (retention_class IN ('standard', 'security', 'compliance', 'restricted')),
  ADD COLUMN IF NOT EXISTS sensitivity TEXT NOT NULL DEFAULT 'standard'
    CHECK (sensitivity IN ('standard', 'restricted')),
  ADD COLUMN IF NOT EXISTS integrity_prev_hash TEXT,
  ADD COLUMN IF NOT EXISTS integrity_hash TEXT;

CREATE INDEX IF NOT EXISTS audit_entries_module_idx
  ON core.audit_entries (tenant_id, module_key, occurred_at DESC);
CREATE INDEX IF NOT EXISTS audit_entries_project_idx
  ON core.audit_entries (tenant_id, project_id, occurred_at DESC)
  WHERE project_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS audit_entries_company_idx
  ON core.audit_entries (tenant_id, company_id, occurred_at DESC)
  WHERE company_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS audit_entries_action_idx
  ON core.audit_entries (tenant_id, action, occurred_at DESC);
CREATE INDEX IF NOT EXISTS audit_entries_actor_idx
  ON core.audit_entries (tenant_id, actor_user_id, occurred_at DESC)
  WHERE actor_user_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS audit_entries_workflow_idx
  ON core.audit_entries (tenant_id, workflow_instance_id)
  WHERE workflow_instance_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS audit_entries_signature_idx
  ON core.audit_entries (tenant_id, signature_package_id)
  WHERE signature_package_id IS NOT NULL;

-- Retention policy per tenant (advisory for export/archive jobs)
CREATE TABLE IF NOT EXISTS core.audit_retention_policies (
  tenant_id           UUID PRIMARY KEY,
  standard_days       INT NOT NULL DEFAULT 2555,
  security_days       INT NOT NULL DEFAULT 2555,
  compliance_days     INT NOT NULL DEFAULT 2555,
  restricted_days     INT NOT NULL DEFAULT 3650,
  export_before_purge BOOLEAN NOT NULL DEFAULT true,
  updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Export jobs (artifact metadata; bytes live in object storage later)
CREATE TABLE IF NOT EXISTS core.audit_export_jobs (
  id                UUID PRIMARY KEY,
  tenant_id         UUID NOT NULL,
  requested_by      UUID,
  status            TEXT NOT NULL CHECK (status IN ('queued', 'running', 'completed', 'failed')),
  filter            JSONB NOT NULL DEFAULT '{}'::jsonb,
  entry_count       INT,
  storage_key       TEXT,
  error_message     TEXT,
  created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
  completed_at      TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS audit_export_jobs_tenant_idx
  ON core.audit_export_jobs (tenant_id, created_at DESC);

ALTER TABLE core.audit_retention_policies ENABLE ROW LEVEL SECURITY;
ALTER TABLE core.audit_export_jobs ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS audit_retention_policies_isolation ON core.audit_retention_policies;
CREATE POLICY audit_retention_policies_isolation ON core.audit_retention_policies
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));

DROP POLICY IF EXISTS audit_export_jobs_isolation ON core.audit_export_jobs;
CREATE POLICY audit_export_jobs_isolation ON core.audit_export_jobs
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));
