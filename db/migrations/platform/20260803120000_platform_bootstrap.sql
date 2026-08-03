-- Foundation bootstrap only.
-- Creates the platform schema placeholder.
-- Business tables are intentionally omitted.
-- Migration metadata is tracked by sqlx in public._sqlx_migrations.

CREATE SCHEMA IF NOT EXISTS platform;

COMMENT ON SCHEMA platform IS
  'Platform-owned schema. Application domain tables are added in later migrations.';
