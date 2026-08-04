-- Enterprise RBAC expansions (ADR-0007).
-- Extends role kinds, grant scopes, permission overrides, and module permission catalog.

-- Expand roles.kind
ALTER TABLE core.roles DROP CONSTRAINT IF EXISTS roles_kind_check;
ALTER TABLE core.roles ADD CONSTRAINT roles_kind_check
  CHECK (kind IN ('system', 'tenant_custom', 'membership', 'company', 'project', 'temporary'));

-- Expand access_grants.scope_type with company
ALTER TABLE core.access_grants DROP CONSTRAINT IF EXISTS access_grants_scope_type_check;
ALTER TABLE core.access_grants ADD CONSTRAINT access_grants_scope_type_check
  CHECK (scope_type IN ('tenant', 'org_unit', 'company', 'project', 'team', 'self'));

-- Permission metadata for families / ABAC readiness
ALTER TABLE core.permissions
  ADD COLUMN IF NOT EXISTS family TEXT NOT NULL DEFAULT 'core',
  ADD COLUMN IF NOT EXISTS sensitivity TEXT NOT NULL DEFAULT 'standard'
    CHECK (sensitivity IN ('standard', 'elevated', 'break_glass'));

-- Permission overrides (deny wins over allow)
CREATE TABLE IF NOT EXISTS core.permission_overrides (
  id                UUID PRIMARY KEY,
  tenant_id         UUID NOT NULL REFERENCES core.tenants (id),
  user_id           UUID NOT NULL REFERENCES core.users (id),
  permission_code   TEXT NOT NULL REFERENCES core.permissions (code),
  effect            TEXT NOT NULL CHECK (effect IN ('allow', 'deny')),
  scope_type        TEXT NOT NULL CHECK (scope_type IN ('tenant', 'org_unit', 'company', 'project', 'team', 'self')),
  scope_id          UUID,
  reason            TEXT,
  expires_at        TIMESTAMPTZ,
  revoked_at        TIMESTAMPTZ,
  created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
  created_by        UUID
);

CREATE INDEX IF NOT EXISTS permission_overrides_user_idx
  ON core.permission_overrides (tenant_id, user_id)
  WHERE revoked_at IS NULL;

ALTER TABLE core.permission_overrides ENABLE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS permission_overrides_isolation ON core.permission_overrides;
CREATE POLICY permission_overrides_isolation ON core.permission_overrides
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));
