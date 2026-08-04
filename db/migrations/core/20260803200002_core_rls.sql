-- Enable Row Level Security on tenant-scoped Core tables.
-- Application sets: SELECT set_config('app.tenant_id', '<uuid>', true);

ALTER TABLE core.tenants ENABLE ROW LEVEL SECURITY;
ALTER TABLE core.companies ENABLE ROW LEVEL SECURITY;
ALTER TABLE core.org_units ENABLE ROW LEVEL SECURITY;
ALTER TABLE core.users ENABLE ROW LEVEL SECURITY;
ALTER TABLE core.sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE core.access_grants ENABLE ROW LEVEL SECURITY;
ALTER TABLE core.project_memberships ENABLE ROW LEVEL SECURITY;
ALTER TABLE core.teams ENABLE ROW LEVEL SECURITY;
ALTER TABLE core.file_objects ENABLE ROW LEVEL SECURITY;
ALTER TABLE core.audit_entries ENABLE ROW LEVEL SECURITY;
ALTER TABLE core.settings_bundles ENABLE ROW LEVEL SECURITY;
ALTER TABLE core.licenses ENABLE ROW LEVEL SECURITY;

-- Tenant isolation policies (permissive for migrator/superuser; app role must use GUC).
CREATE POLICY tenants_isolation ON core.tenants
  USING (id::text = nullif(current_setting('app.tenant_id', true), ''));

CREATE POLICY companies_isolation ON core.companies
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));

CREATE POLICY org_units_isolation ON core.org_units
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));

CREATE POLICY users_isolation ON core.users
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));

CREATE POLICY sessions_isolation ON core.sessions
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));

CREATE POLICY access_grants_isolation ON core.access_grants
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));

CREATE POLICY project_memberships_isolation ON core.project_memberships
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));

CREATE POLICY teams_isolation ON core.teams
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));

CREATE POLICY file_objects_isolation ON core.file_objects
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));

CREATE POLICY audit_entries_isolation ON core.audit_entries
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));

CREATE POLICY settings_bundles_isolation ON core.settings_bundles
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));

CREATE POLICY licenses_isolation ON core.licenses
  USING (tenant_id::text = nullif(current_setting('app.tenant_id', true), ''));
