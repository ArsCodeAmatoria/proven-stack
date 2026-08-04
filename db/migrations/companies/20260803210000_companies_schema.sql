-- Companies profile schema (ADR-0005).
-- Extends Core CompanyId — no FK to core.companies; UUID references only.
-- Does not create projects or other business-module tables.

CREATE SCHEMA IF NOT EXISTS companies;

COMMENT ON SCHEMA companies IS
  'Company profile & configuration. Legal company identity remains in core.companies.';

CREATE TABLE companies.company_profiles (
  company_id      UUID PRIMARY KEY,
  tenant_id       UUID NOT NULL,
  status          TEXT NOT NULL CHECK (status IN ('active', 'archived')),
  trade_name      TEXT,
  website         TEXT,
  notes           TEXT,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  version         BIGINT NOT NULL DEFAULT 1
);

CREATE INDEX company_profiles_tenant_idx ON companies.company_profiles (tenant_id);

-- Business units (company-scoped hierarchy; optional Core OrgUnit link)
CREATE TABLE companies.business_units (
  id              UUID PRIMARY KEY,
  tenant_id       UUID NOT NULL,
  company_id      UUID NOT NULL REFERENCES companies.company_profiles (company_id) ON DELETE CASCADE,
  parent_id       UUID REFERENCES companies.business_units (id),
  org_unit_id     UUID,
  name            TEXT NOT NULL,
  code            TEXT,
  status          TEXT NOT NULL CHECK (status IN ('active', 'archived')),
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  version         BIGINT NOT NULL DEFAULT 1
);

CREATE INDEX business_units_company_idx ON companies.business_units (tenant_id, company_id);
CREATE UNIQUE INDEX business_units_company_code_uidx
  ON companies.business_units (company_id, lower(code))
  WHERE code IS NOT NULL AND status = 'active';

CREATE TABLE companies.addresses (
  id              UUID PRIMARY KEY,
  tenant_id       UUID NOT NULL,
  company_id      UUID NOT NULL REFERENCES companies.company_profiles (company_id) ON DELETE CASCADE,
  business_unit_id UUID REFERENCES companies.business_units (id) ON DELETE SET NULL,
  kind            TEXT NOT NULL CHECK (kind IN ('head_office', 'billing', 'site', 'mailing', 'other')),
  line1           TEXT NOT NULL,
  line2           TEXT,
  city            TEXT NOT NULL,
  region          TEXT,
  postal_code     TEXT,
  country_code    TEXT NOT NULL,
  is_primary      BOOLEAN NOT NULL DEFAULT false,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX addresses_company_idx ON companies.addresses (tenant_id, company_id);

CREATE TABLE companies.contacts (
  id              UUID PRIMARY KEY,
  tenant_id       UUID NOT NULL,
  company_id      UUID NOT NULL REFERENCES companies.company_profiles (company_id) ON DELETE CASCADE,
  business_unit_id UUID REFERENCES companies.business_units (id) ON DELETE SET NULL,
  kind            TEXT NOT NULL CHECK (kind IN ('primary', 'billing', 'safety', 'hr', 'operations', 'other')),
  full_name       TEXT NOT NULL,
  title           TEXT,
  email           TEXT,
  phone           TEXT,
  user_id         UUID,
  is_primary      BOOLEAN NOT NULL DEFAULT false,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX contacts_company_idx ON companies.contacts (tenant_id, company_id);

CREATE TABLE companies.branding (
  company_id          UUID PRIMARY KEY REFERENCES companies.company_profiles (company_id) ON DELETE CASCADE,
  tenant_id           UUID NOT NULL,
  logo_file_id        UUID,
  wordmark_file_id    UUID,
  primary_color       TEXT,
  secondary_color     TEXT,
  accent_color        TEXT,
  favicon_file_id     UUID,
  updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE companies.safety_settings (
  company_id                      UUID PRIMARY KEY REFERENCES companies.company_profiles (company_id) ON DELETE CASCADE,
  tenant_id                       UUID NOT NULL,
  require_flha_before_work        BOOLEAN NOT NULL DEFAULT true,
  require_toolbox_talk_weekly     BOOLEAN NOT NULL DEFAULT false,
  incident_notify_emails          JSONB NOT NULL DEFAULT '[]'::jsonb,
  default_risk_matrix             TEXT NOT NULL DEFAULT 'standard',
  allow_offline_safety_submit     BOOLEAN NOT NULL DEFAULT true,
  updated_at                      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE companies.regional_settings (
  company_id          UUID PRIMARY KEY REFERENCES companies.company_profiles (company_id) ON DELETE CASCADE,
  tenant_id           UUID NOT NULL,
  primary_region      TEXT NOT NULL,
  locales             JSONB NOT NULL DEFAULT '[]'::jsonb,
  timezone            TEXT NOT NULL DEFAULT 'UTC',
  measurement_system  TEXT NOT NULL DEFAULT 'metric'
    CHECK (measurement_system IN ('metric', 'imperial')),
  currency_code       TEXT NOT NULL DEFAULT 'CAD',
  updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Pointers to templates owned by other modules (ids only — no template bodies here).
CREATE TABLE companies.default_templates (
  id              UUID PRIMARY KEY,
  tenant_id       UUID NOT NULL,
  company_id      UUID NOT NULL REFERENCES companies.company_profiles (company_id) ON DELETE CASCADE,
  template_kind   TEXT NOT NULL CHECK (
    template_kind IN (
      'project', 'flha', 'inspection', 'toolbox', 'document', 'training', 'notification', 'other'
    )
  ),
  template_ref    TEXT NOT NULL,
  label           TEXT,
  is_default      BOOLEAN NOT NULL DEFAULT true,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX default_templates_kind_uidx
  ON companies.default_templates (company_id, template_kind)
  WHERE is_default;

CREATE TABLE companies.notification_defaults (
  company_id              UUID PRIMARY KEY REFERENCES companies.company_profiles (company_id) ON DELETE CASCADE,
  tenant_id               UUID NOT NULL,
  email_enabled           BOOLEAN NOT NULL DEFAULT true,
  push_enabled            BOOLEAN NOT NULL DEFAULT true,
  sms_enabled             BOOLEAN NOT NULL DEFAULT false,
  digest_cadence          TEXT NOT NULL DEFAULT 'daily'
    CHECK (digest_cadence IN ('realtime', 'hourly', 'daily', 'weekly', 'off')),
  quiet_hours_start       TEXT,
  quiet_hours_end         TEXT,
  default_locale          TEXT NOT NULL DEFAULT 'en',
  updated_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE companies.storage_configuration (
  company_id              UUID PRIMARY KEY REFERENCES companies.company_profiles (company_id) ON DELETE CASCADE,
  tenant_id               UUID NOT NULL,
  object_prefix           TEXT NOT NULL,
  max_upload_bytes        BIGINT NOT NULL DEFAULT 52428800,
  allowed_content_types   JSONB NOT NULL DEFAULT '["application/pdf","image/jpeg","image/png"]'::jsonb,
  retention_class_default TEXT NOT NULL DEFAULT 'standard',
  quarantine_enabled      BOOLEAN NOT NULL DEFAULT true,
  updated_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- RLS
ALTER TABLE companies.company_profiles ENABLE ROW LEVEL SECURITY;
ALTER TABLE companies.business_units ENABLE ROW LEVEL SECURITY;
ALTER TABLE companies.addresses ENABLE ROW LEVEL SECURITY;
ALTER TABLE companies.contacts ENABLE ROW LEVEL SECURITY;
ALTER TABLE companies.branding ENABLE ROW LEVEL SECURITY;
ALTER TABLE companies.safety_settings ENABLE ROW LEVEL SECURITY;
ALTER TABLE companies.regional_settings ENABLE ROW LEVEL SECURITY;
ALTER TABLE companies.default_templates ENABLE ROW LEVEL SECURITY;
ALTER TABLE companies.notification_defaults ENABLE ROW LEVEL SECURITY;
ALTER TABLE companies.storage_configuration ENABLE ROW LEVEL SECURITY;

CREATE POLICY company_profiles_isolation ON companies.company_profiles
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));
CREATE POLICY business_units_isolation ON companies.business_units
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));
CREATE POLICY addresses_isolation ON companies.addresses
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));
CREATE POLICY contacts_isolation ON companies.contacts
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));
CREATE POLICY branding_isolation ON companies.branding
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));
CREATE POLICY safety_settings_isolation ON companies.safety_settings
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));
CREATE POLICY regional_settings_isolation ON companies.regional_settings
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));
CREATE POLICY default_templates_isolation ON companies.default_templates
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));
CREATE POLICY notification_defaults_isolation ON companies.notification_defaults
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));
CREATE POLICY storage_configuration_isolation ON companies.storage_configuration
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));
