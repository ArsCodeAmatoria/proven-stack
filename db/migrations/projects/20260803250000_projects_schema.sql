-- Projects Place schema skeleton (ADR-0009).
-- Owns project lifecycle, primary location, and company participants.
-- Worker ACL remains in core.project_memberships — UUID refs only, no cross-schema FKs.
-- No safety / inspections / forms tables in this skeleton.

CREATE SCHEMA IF NOT EXISTS projects;

COMMENT ON SCHEMA projects IS
  'Project Place aggregate. Membership ACL remains in core; Equipment/Safety/Documents are other modules.';

CREATE TABLE projects.projects (
  id                          UUID PRIMARY KEY,
  tenant_id                   UUID NOT NULL,
  code                        TEXT NOT NULL,
  name                        TEXT NOT NULL,
  description                 TEXT,
  status                      TEXT NOT NULL CHECK (status IN (
                                'planning', 'active', 'on_hold', 'closed', 'archived'
                              )),
  -- Primary location (areas deferred)
  location_line1              TEXT,
  location_line2              TEXT,
  location_city               TEXT,
  location_region             TEXT,
  location_postal_code         TEXT,
  location_country_code       TEXT,
  location_timezone           TEXT,
  prime_contractor_company_id UUID NOT NULL,
  client_company_id           UUID,
  planned_start               DATE,
  planned_end                 DATE,
  created_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
  version                     BIGINT NOT NULL DEFAULT 1
);

CREATE UNIQUE INDEX projects_tenant_code_uidx
  ON projects.projects (tenant_id, lower(code))
  WHERE status <> 'archived';

CREATE INDEX projects_tenant_status_idx
  ON projects.projects (tenant_id, status);

CREATE INDEX projects_tenant_updated_idx
  ON projects.projects (tenant_id, updated_at DESC);

CREATE TABLE projects.project_participants (
  id              UUID PRIMARY KEY,
  tenant_id       UUID NOT NULL,
  project_id      UUID NOT NULL REFERENCES projects.projects (id) ON DELETE CASCADE,
  company_id      UUID NOT NULL,
  role            TEXT NOT NULL CHECK (role IN (
                    'prime', 'subcontractor', 'client', 'supplier', 'other'
                  )),
  status          TEXT NOT NULL CHECK (status IN (
                    'invited', 'active', 'suspended', 'removed'
                  )),
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  version         BIGINT NOT NULL DEFAULT 1
);

CREATE UNIQUE INDEX project_participants_active_uidx
  ON projects.project_participants (project_id, company_id, role)
  WHERE status IN ('invited', 'active');

CREATE INDEX project_participants_company_idx
  ON projects.project_participants (tenant_id, company_id);

-- Settings shell placeholder (no API in skeleton).
CREATE TABLE projects.project_settings (
  project_id   UUID PRIMARY KEY REFERENCES projects.projects (id) ON DELETE CASCADE,
  tenant_id    UUID NOT NULL,
  settings     JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  version      BIGINT NOT NULL DEFAULT 1
);

COMMENT ON TABLE projects.project_settings IS
  'Placeholder for project-scoped settings; Settings API is deferred (ADR-0009).';
