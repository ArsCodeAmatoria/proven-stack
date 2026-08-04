-- File management engine expansion (ADR-0010).
-- Extends core.file_objects with class, metadata, versioning, temporary uploads, scan status.
-- Adds core.file_share_links for API-mediated public share tokens (R2 bucket stays private).

ALTER TABLE core.file_objects
  DROP CONSTRAINT IF EXISTS file_objects_status_check;

ALTER TABLE core.file_objects
  ADD CONSTRAINT file_objects_status_check
  CHECK (status IN (
    'pending_upload', 'processing', 'available', 'quarantined', 'deleted'
  ));

ALTER TABLE core.file_objects
  ADD COLUMN IF NOT EXISTS object_class TEXT NOT NULL DEFAULT 'attachment'
    CHECK (object_class IN (
      'photo', 'pdf', 'video', 'certificate', 'drawing', 'attachment'
    )),
  ADD COLUMN IF NOT EXISTS original_filename TEXT,
  ADD COLUMN IF NOT EXISTS metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  ADD COLUMN IF NOT EXISTS parent_file_id UUID REFERENCES core.file_objects (id),
  ADD COLUMN IF NOT EXISTS content_version INT NOT NULL DEFAULT 1,
  ADD COLUMN IF NOT EXISTS is_temporary BOOLEAN NOT NULL DEFAULT false,
  ADD COLUMN IF NOT EXISTS expires_at TIMESTAMPTZ,
  ADD COLUMN IF NOT EXISTS scan_status TEXT NOT NULL DEFAULT 'not_scanned'
    CHECK (scan_status IN (
      'not_scanned', 'pending', 'clean', 'infected', 'error'
    )),
  ADD COLUMN IF NOT EXISTS scan_detail TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS file_objects_storage_key_uidx
  ON core.file_objects (storage_key);

CREATE INDEX IF NOT EXISTS file_objects_parent_idx
  ON core.file_objects (tenant_id, parent_file_id)
  WHERE parent_file_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS file_objects_temporary_expiry_idx
  ON core.file_objects (tenant_id, expires_at)
  WHERE is_temporary AND status <> 'deleted';

CREATE INDEX IF NOT EXISTS file_objects_class_idx
  ON core.file_objects (tenant_id, object_class, status);

CREATE TABLE IF NOT EXISTS core.file_share_links (
  id              UUID PRIMARY KEY,
  tenant_id       UUID NOT NULL REFERENCES core.tenants (id),
  file_id         UUID NOT NULL REFERENCES core.file_objects (id),
  token           TEXT NOT NULL UNIQUE,
  kind            TEXT NOT NULL CHECK (kind IN ('private', 'public_share')),
  expires_at      TIMESTAMPTZ NOT NULL,
  created_by      UUID REFERENCES core.users (id),
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  revoked_at      TIMESTAMPTZ,
  max_downloads   INT,
  download_count  INT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS file_share_links_file_idx
  ON core.file_share_links (tenant_id, file_id);

COMMENT ON TABLE core.file_share_links IS
  'API-mediated share tokens. R2 buckets remain private — never anonymous public ACLs (ADR-0010).';
