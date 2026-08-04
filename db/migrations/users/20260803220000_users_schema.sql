-- Users account profile schema (ADR-0006).
-- Extends Core UserId — no FK to core.users; UUID references only.
-- Does not implement project assignments or People workforce SoR.

CREATE SCHEMA IF NOT EXISTS users;

COMMENT ON SCHEMA users IS
  'Account profile & preferences. Login identity remains in core.users.';

CREATE TABLE users.user_profiles (
  user_id           UUID PRIMARY KEY,
  tenant_id         UUID NOT NULL,
  status            TEXT NOT NULL CHECK (status IN ('active', 'archived')),
  display_name      TEXT NOT NULL,
  preferred_name    TEXT,
  job_title         TEXT,
  phone             TEXT,
  company_id        UUID,
  person_id         UUID,
  bio               TEXT,
  created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
  version           BIGINT NOT NULL DEFAULT 1
);

CREATE INDEX user_profiles_tenant_idx ON users.user_profiles (tenant_id);

-- Platform classifications (not Core RBAC, not People workforce roles)
CREATE TABLE users.user_kinds (
  id              UUID PRIMARY KEY,
  tenant_id       UUID NOT NULL,
  user_id         UUID NOT NULL REFERENCES users.user_profiles (user_id) ON DELETE CASCADE,
  kind            TEXT NOT NULL CHECK (kind IN (
    'worker', 'supervisor', 'manager', 'safety_coordinator',
    'administrator', 'external', 'guest'
  )),
  is_primary      BOOLEAN NOT NULL DEFAULT false,
  assigned_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_id, kind)
);

CREATE INDEX user_kinds_tenant_kind_idx ON users.user_kinds (tenant_id, kind);

CREATE TABLE users.avatars (
  user_id           UUID PRIMARY KEY REFERENCES users.user_profiles (user_id) ON DELETE CASCADE,
  tenant_id         UUID NOT NULL,
  file_object_id    UUID,
  avatar_url        TEXT,
  updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE users.locale_preferences (
  user_id           UUID PRIMARY KEY REFERENCES users.user_profiles (user_id) ON DELETE CASCADE,
  tenant_id         UUID NOT NULL,
  language_code     TEXT NOT NULL DEFAULT 'en',
  time_zone         TEXT NOT NULL DEFAULT 'UTC',
  updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE users.accessibility_preferences (
  user_id               UUID PRIMARY KEY REFERENCES users.user_profiles (user_id) ON DELETE CASCADE,
  tenant_id             UUID NOT NULL,
  reduce_motion         BOOLEAN NOT NULL DEFAULT false,
  high_contrast         BOOLEAN NOT NULL DEFAULT false,
  large_text            BOOLEAN NOT NULL DEFAULT false,
  screen_reader_hints   BOOLEAN NOT NULL DEFAULT false,
  updated_at            TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE users.notification_preferences (
  user_id               UUID PRIMARY KEY REFERENCES users.user_profiles (user_id) ON DELETE CASCADE,
  tenant_id             UUID NOT NULL,
  email_enabled         BOOLEAN NOT NULL DEFAULT true,
  push_enabled          BOOLEAN NOT NULL DEFAULT true,
  sms_enabled           BOOLEAN NOT NULL DEFAULT false,
  in_app_enabled        BOOLEAN NOT NULL DEFAULT true,
  digest_cadence        TEXT NOT NULL DEFAULT 'daily'
    CHECK (digest_cadence IN ('realtime', 'hourly', 'daily', 'weekly', 'off')),
  quiet_hours_start     TEXT,
  quiet_hours_end       TEXT,
  updated_at            TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Auth preference / mirror flags (never password hashes — Core owns credentials)
CREATE TABLE users.authentication_profiles (
  user_id                   UUID PRIMARY KEY REFERENCES users.user_profiles (user_id) ON DELETE CASCADE,
  tenant_id                 UUID NOT NULL,
  mfa_preferred             BOOLEAN NOT NULL DEFAULT false,
  password_login_enabled    BOOLEAN NOT NULL DEFAULT true,
  oauth_google_linked       BOOLEAN NOT NULL DEFAULT false,
  oauth_microsoft_linked    BOOLEAN NOT NULL DEFAULT false,
  magic_link_preferred      BOOLEAN NOT NULL DEFAULT false,
  last_auth_method          TEXT,
  updated_at                TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE users.digital_signature_profiles (
  user_id                   UUID PRIMARY KEY REFERENCES users.user_profiles (user_id) ON DELETE CASCADE,
  tenant_id                 UUID NOT NULL,
  default_signature_type    TEXT NOT NULL DEFAULT 'drawn'
    CHECK (default_signature_type IN ('drawn', 'typed', 'uploaded', 'clickwrap')),
  typed_name_default        TEXT,
  signature_image_file_id   UUID,
  require_reauth_to_sign    BOOLEAN NOT NULL DEFAULT false,
  updated_at                TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE users.emergency_contacts (
  id              UUID PRIMARY KEY,
  tenant_id       UUID NOT NULL,
  user_id         UUID NOT NULL REFERENCES users.user_profiles (user_id) ON DELETE CASCADE,
  full_name       TEXT NOT NULL,
  relationship    TEXT,
  phone           TEXT NOT NULL,
  email           TEXT,
  is_primary      BOOLEAN NOT NULL DEFAULT false,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX emergency_contacts_user_idx ON users.emergency_contacts (tenant_id, user_id);

CREATE TABLE users.user_settings (
  user_id         UUID NOT NULL REFERENCES users.user_profiles (user_id) ON DELETE CASCADE,
  tenant_id       UUID NOT NULL,
  key             TEXT NOT NULL,
  value           JSONB NOT NULL,
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (user_id, key)
);

-- Append-only profile change history (not a substitute for core.audit_entries)
CREATE TABLE users.profile_audit_entries (
  id              UUID PRIMARY KEY,
  tenant_id       UUID NOT NULL,
  user_id         UUID NOT NULL,
  actor_user_id   UUID,
  action          TEXT NOT NULL,
  resource_type   TEXT NOT NULL,
  resource_id     UUID,
  summary         TEXT NOT NULL,
  payload         JSONB NOT NULL DEFAULT '{}'::jsonb,
  occurred_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX profile_audit_user_time_idx
  ON users.profile_audit_entries (tenant_id, user_id, occurred_at DESC);

-- RLS
ALTER TABLE users.user_profiles ENABLE ROW LEVEL SECURITY;
ALTER TABLE users.user_kinds ENABLE ROW LEVEL SECURITY;
ALTER TABLE users.avatars ENABLE ROW LEVEL SECURITY;
ALTER TABLE users.locale_preferences ENABLE ROW LEVEL SECURITY;
ALTER TABLE users.accessibility_preferences ENABLE ROW LEVEL SECURITY;
ALTER TABLE users.notification_preferences ENABLE ROW LEVEL SECURITY;
ALTER TABLE users.authentication_profiles ENABLE ROW LEVEL SECURITY;
ALTER TABLE users.digital_signature_profiles ENABLE ROW LEVEL SECURITY;
ALTER TABLE users.emergency_contacts ENABLE ROW LEVEL SECURITY;
ALTER TABLE users.user_settings ENABLE ROW LEVEL SECURITY;
ALTER TABLE users.profile_audit_entries ENABLE ROW LEVEL SECURITY;

CREATE POLICY user_profiles_isolation ON users.user_profiles
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));
CREATE POLICY user_kinds_isolation ON users.user_kinds
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));
CREATE POLICY avatars_isolation ON users.avatars
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));
CREATE POLICY locale_preferences_isolation ON users.locale_preferences
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));
CREATE POLICY accessibility_preferences_isolation ON users.accessibility_preferences
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));
CREATE POLICY notification_preferences_isolation ON users.notification_preferences
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));
CREATE POLICY authentication_profiles_isolation ON users.authentication_profiles
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));
CREATE POLICY digital_signature_profiles_isolation ON users.digital_signature_profiles
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));
CREATE POLICY emergency_contacts_isolation ON users.emergency_contacts
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));
CREATE POLICY user_settings_isolation ON users.user_settings
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));
CREATE POLICY profile_audit_entries_isolation ON users.profile_audit_entries
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));
