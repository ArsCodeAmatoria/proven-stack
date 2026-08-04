-- Core schema: platform foundation (tenancy, identity, access, membership, files, audit, settings, flags, licensing).
-- No cross-schema foreign keys. See docs/adr/0004-core-persistence.md and CORE_DOMAIN.md.

CREATE SCHEMA IF NOT EXISTS core;

COMMENT ON SCHEMA core IS
  'Core platform foundation. Other modules must not read/write these tables directly.';

-- ---------------------------------------------------------------------------
-- Tenancy
-- ---------------------------------------------------------------------------

CREATE TABLE core.tenants (
  id            UUID PRIMARY KEY,
  slug          TEXT NOT NULL UNIQUE,
  display_name  TEXT NOT NULL,
  region_code   TEXT NOT NULL,
  status        TEXT NOT NULL CHECK (status IN ('active', 'suspended', 'closed')),
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  version       BIGINT NOT NULL DEFAULT 1
);

CREATE TABLE core.companies (
  id            UUID PRIMARY KEY,
  tenant_id     UUID NOT NULL REFERENCES core.tenants (id),
  legal_name    TEXT NOT NULL,
  display_name  TEXT NOT NULL,
  company_type  TEXT NOT NULL CHECK (
    company_type IN ('prime', 'subcontractor', 'crane', 'forming', 'civil', 'industrial', 'other')
  ),
  status        TEXT NOT NULL CHECK (status IN ('active', 'deactivated')),
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  version       BIGINT NOT NULL DEFAULT 1
);

CREATE INDEX companies_tenant_idx ON core.companies (tenant_id);
CREATE UNIQUE INDEX companies_tenant_legal_name_uidx
  ON core.companies (tenant_id, lower(legal_name))
  WHERE status = 'active';

CREATE TABLE core.org_units (
  id            UUID PRIMARY KEY,
  tenant_id     UUID NOT NULL REFERENCES core.tenants (id),
  parent_id     UUID REFERENCES core.org_units (id),
  name          TEXT NOT NULL,
  status        TEXT NOT NULL CHECK (status IN ('active', 'archived')),
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  version       BIGINT NOT NULL DEFAULT 1
);

CREATE INDEX org_units_tenant_idx ON core.org_units (tenant_id);
CREATE INDEX org_units_parent_idx ON core.org_units (tenant_id, parent_id);

-- ---------------------------------------------------------------------------
-- Identity
-- ---------------------------------------------------------------------------

CREATE TABLE core.users (
  id            UUID PRIMARY KEY,
  tenant_id     UUID NOT NULL REFERENCES core.tenants (id),
  email         TEXT NOT NULL,
  display_name  TEXT NOT NULL,
  status        TEXT NOT NULL CHECK (status IN ('invited', 'active', 'locked', 'deactivated')),
  person_id     UUID,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  version       BIGINT NOT NULL DEFAULT 1
);

CREATE UNIQUE INDEX users_tenant_email_uidx ON core.users (tenant_id, lower(email));
CREATE INDEX users_person_idx ON core.users (tenant_id, person_id);

CREATE TABLE core.credentials (
  id              UUID PRIMARY KEY,
  user_id         UUID NOT NULL REFERENCES core.users (id) ON DELETE CASCADE,
  credential_type TEXT NOT NULL CHECK (credential_type IN ('password', 'webauthn')),
  secret_hash     TEXT NOT NULL,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_id, credential_type)
);

CREATE TABLE core.external_identity_links (
  id              UUID PRIMARY KEY,
  user_id         UUID NOT NULL REFERENCES core.users (id) ON DELETE CASCADE,
  provider        TEXT NOT NULL,
  subject         TEXT NOT NULL,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (provider, subject)
);

CREATE TABLE core.sessions (
  id              UUID PRIMARY KEY,
  tenant_id       UUID NOT NULL REFERENCES core.tenants (id),
  user_id         UUID NOT NULL REFERENCES core.users (id) ON DELETE CASCADE,
  status          TEXT NOT NULL CHECK (status IN ('active', 'revoked')),
  expires_at      TIMESTAMPTZ NOT NULL,
  revoked_at      TIMESTAMPTZ,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  ip_address      TEXT,
  user_agent      TEXT
);

CREATE INDEX sessions_user_idx ON core.sessions (tenant_id, user_id);
CREATE INDEX sessions_active_idx ON core.sessions (id) WHERE status = 'active';

-- ---------------------------------------------------------------------------
-- Access control
-- ---------------------------------------------------------------------------

CREATE TABLE core.permissions (
  code          TEXT PRIMARY KEY,
  description   TEXT NOT NULL,
  module_key    TEXT NOT NULL,
  retired       BOOLEAN NOT NULL DEFAULT false,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE core.roles (
  id            UUID PRIMARY KEY,
  tenant_id     UUID REFERENCES core.tenants (id),
  name          TEXT NOT NULL,
  kind          TEXT NOT NULL CHECK (kind IN ('system', 'tenant_custom', 'membership')),
  status        TEXT NOT NULL CHECK (status IN ('active', 'retired')),
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  version       BIGINT NOT NULL DEFAULT 1
);

CREATE UNIQUE INDEX roles_system_name_uidx ON core.roles (lower(name)) WHERE tenant_id IS NULL;
CREATE UNIQUE INDEX roles_tenant_name_uidx ON core.roles (tenant_id, lower(name)) WHERE tenant_id IS NOT NULL;

CREATE TABLE core.role_permissions (
  role_id           UUID NOT NULL REFERENCES core.roles (id) ON DELETE CASCADE,
  permission_code   TEXT NOT NULL REFERENCES core.permissions (code),
  PRIMARY KEY (role_id, permission_code)
);

CREATE TABLE core.access_grants (
  id              UUID PRIMARY KEY,
  tenant_id       UUID NOT NULL REFERENCES core.tenants (id),
  user_id         UUID NOT NULL REFERENCES core.users (id),
  role_id         UUID NOT NULL REFERENCES core.roles (id),
  scope_type      TEXT NOT NULL CHECK (scope_type IN ('tenant', 'org_unit', 'project', 'team', 'self')),
  scope_id        UUID,
  grant_kind      TEXT NOT NULL DEFAULT 'standard'
    CHECK (grant_kind IN ('standard', 'delegation', 'temporary', 'break_glass')),
  expires_at      TIMESTAMPTZ,
  revoked_at      TIMESTAMPTZ,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  created_by      UUID
);

CREATE INDEX access_grants_user_idx ON core.access_grants (tenant_id, user_id);
CREATE INDEX access_grants_scope_idx ON core.access_grants (tenant_id, scope_type, scope_id);

-- ---------------------------------------------------------------------------
-- Membership & teams
-- ---------------------------------------------------------------------------

CREATE TABLE core.project_memberships (
  id              UUID PRIMARY KEY,
  tenant_id       UUID NOT NULL REFERENCES core.tenants (id),
  project_id      UUID NOT NULL,
  user_id         UUID REFERENCES core.users (id),
  person_id       UUID,
  membership_role TEXT NOT NULL,
  status          TEXT NOT NULL CHECK (status IN ('invited', 'active', 'suspended', 'removed')),
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  version         BIGINT NOT NULL DEFAULT 1,
  CONSTRAINT project_memberships_subject_chk CHECK (user_id IS NOT NULL OR person_id IS NOT NULL)
);

CREATE UNIQUE INDEX project_memberships_active_uidx
  ON core.project_memberships (tenant_id, project_id, COALESCE(person_id, user_id))
  WHERE status IN ('invited', 'active', 'suspended');

CREATE INDEX project_memberships_project_idx ON core.project_memberships (tenant_id, project_id);
CREATE INDEX project_memberships_user_idx ON core.project_memberships (tenant_id, user_id);

CREATE TABLE core.teams (
  id              UUID PRIMARY KEY,
  tenant_id       UUID NOT NULL REFERENCES core.tenants (id),
  name            TEXT NOT NULL,
  project_id      UUID,
  status          TEXT NOT NULL CHECK (status IN ('active', 'archived')),
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  version         BIGINT NOT NULL DEFAULT 1
);

CREATE INDEX teams_tenant_idx ON core.teams (tenant_id);

CREATE TABLE core.team_members (
  team_id         UUID NOT NULL REFERENCES core.teams (id) ON DELETE CASCADE,
  user_id         UUID REFERENCES core.users (id),
  person_id       UUID,
  added_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT team_members_subject_chk CHECK (user_id IS NOT NULL OR person_id IS NOT NULL)
);

CREATE UNIQUE INDEX team_members_uidx
  ON core.team_members (team_id, COALESCE(person_id, user_id));

-- ---------------------------------------------------------------------------
-- Files (metadata only — bytes in object storage)
-- ---------------------------------------------------------------------------

CREATE TABLE core.file_objects (
  id                UUID PRIMARY KEY,
  tenant_id         UUID NOT NULL REFERENCES core.tenants (id),
  status            TEXT NOT NULL CHECK (
    status IN ('pending_upload', 'available', 'quarantined', 'deleted')
  ),
  storage_key       TEXT NOT NULL,
  content_type      TEXT,
  byte_size         BIGINT,
  checksum_sha256   TEXT,
  retention_class   TEXT NOT NULL DEFAULT 'standard',
  access_class      TEXT NOT NULL DEFAULT 'tenant',
  created_by        UUID,
  created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
  version           BIGINT NOT NULL DEFAULT 1
);

CREATE INDEX file_objects_tenant_idx ON core.file_objects (tenant_id);

-- ---------------------------------------------------------------------------
-- Audit (append-only)
-- ---------------------------------------------------------------------------

CREATE TABLE core.audit_entries (
  id                UUID PRIMARY KEY,
  tenant_id         UUID NOT NULL,
  occurred_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
  actor_user_id     UUID,
  actor_type        TEXT NOT NULL DEFAULT 'user',
  action            TEXT NOT NULL,
  resource_type     TEXT NOT NULL,
  resource_id       UUID,
  correlation_id    UUID,
  causation_id      UUID,
  payload           JSONB NOT NULL DEFAULT '{}'::jsonb,
  payload_digest    TEXT NOT NULL
);

CREATE INDEX audit_entries_tenant_time_idx ON core.audit_entries (tenant_id, occurred_at DESC);
CREATE INDEX audit_entries_resource_idx ON core.audit_entries (tenant_id, resource_type, resource_id);
CREATE INDEX audit_entries_correlation_idx ON core.audit_entries (correlation_id);

-- ---------------------------------------------------------------------------
-- Settings & flags
-- ---------------------------------------------------------------------------

CREATE TABLE core.settings_bundles (
  id            UUID PRIMARY KEY,
  tenant_id     UUID NOT NULL REFERENCES core.tenants (id),
  scope_type    TEXT NOT NULL CHECK (scope_type IN ('tenant', 'org_unit', 'user', 'platform')),
  scope_id      UUID,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX settings_bundles_scope_uidx
  ON core.settings_bundles (tenant_id, scope_type, COALESCE(scope_id, '00000000-0000-0000-0000-000000000000'::uuid));

CREATE TABLE core.setting_entries (
  bundle_id     UUID NOT NULL REFERENCES core.settings_bundles (id) ON DELETE CASCADE,
  key           TEXT NOT NULL,
  value         JSONB NOT NULL,
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (bundle_id, key)
);

CREATE TABLE core.feature_flags (
  key           TEXT PRIMARY KEY,
  description   TEXT NOT NULL,
  default_enabled BOOLEAN NOT NULL DEFAULT false,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE core.feature_flag_overrides (
  id            UUID PRIMARY KEY,
  flag_key      TEXT NOT NULL REFERENCES core.feature_flags (key) ON DELETE CASCADE,
  tenant_id     UUID REFERENCES core.tenants (id),
  user_id       UUID REFERENCES core.users (id),
  enabled       BOOLEAN NOT NULL,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX feature_flag_overrides_flag_idx ON core.feature_flag_overrides (flag_key, tenant_id);

-- ---------------------------------------------------------------------------
-- Licensing
-- ---------------------------------------------------------------------------

CREATE TABLE core.licenses (
  id            UUID PRIMARY KEY,
  tenant_id     UUID NOT NULL REFERENCES core.tenants (id),
  status        TEXT NOT NULL CHECK (status IN ('trial', 'active', 'grace', 'expired', 'suspended')),
  plan_code     TEXT NOT NULL,
  seats_limit   INT NOT NULL DEFAULT 0,
  starts_at     TIMESTAMPTZ NOT NULL,
  ends_at       TIMESTAMPTZ,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  version       BIGINT NOT NULL DEFAULT 1
);

CREATE UNIQUE INDEX licenses_tenant_active_uidx
  ON core.licenses (tenant_id)
  WHERE status IN ('trial', 'active', 'grace');

CREATE TABLE core.module_entitlements (
  license_id    UUID NOT NULL REFERENCES core.licenses (id) ON DELETE CASCADE,
  module_key    TEXT NOT NULL,
  enabled       BOOLEAN NOT NULL DEFAULT true,
  PRIMARY KEY (license_id, module_key)
);

CREATE TABLE core.seat_allocations (
  id            UUID PRIMARY KEY,
  license_id    UUID NOT NULL REFERENCES core.licenses (id) ON DELETE CASCADE,
  user_id       UUID NOT NULL REFERENCES core.users (id),
  seat_type     TEXT NOT NULL DEFAULT 'standard',
  allocated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  released_at   TIMESTAMPTZ
);

CREATE INDEX seat_allocations_license_idx ON core.seat_allocations (license_id)
  WHERE released_at IS NULL;

-- Platform outbox (transport owned by platform schema; created here if missing).
CREATE TABLE IF NOT EXISTS platform.outbox_messages (
  id              UUID PRIMARY KEY,
  tenant_id       UUID,
  aggregate_type  TEXT NOT NULL,
  aggregate_id    UUID NOT NULL,
  event_type      TEXT NOT NULL,
  event_version   INT NOT NULL DEFAULT 1,
  payload         JSONB NOT NULL,
  occurred_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  published_at    TIMESTAMPTZ,
  attempts        INT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS outbox_unpublished_idx
  ON platform.outbox_messages (occurred_at)
  WHERE published_at IS NULL;
