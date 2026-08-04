-- Seed Core permission catalog and system roles (reference data owned by Core).

INSERT INTO core.permissions (code, description, module_key) VALUES
  ('core.tenant.read', 'Read tenant profile', 'core'),
  ('core.tenant.manage', 'Manage tenant lifecycle', 'core'),
  ('core.company.manage', 'Manage companies', 'core'),
  ('core.org.manage', 'Manage org units', 'core'),
  ('core.user.invite', 'Invite users', 'core'),
  ('core.user.manage', 'Manage users', 'core'),
  ('core.role.manage', 'Manage roles', 'core'),
  ('core.grant.manage', 'Manage access grants', 'core'),
  ('core.membership.manage', 'Manage project memberships', 'core'),
  ('core.team.manage', 'Manage teams', 'core'),
  ('core.file.upload', 'Create file upload intents', 'core'),
  ('core.file.read', 'Read file metadata / download', 'core'),
  ('core.file.delete', 'Delete file objects', 'core'),
  ('core.audit.read', 'Query audit entries', 'core'),
  ('core.audit.export', 'Export audit log', 'core'),
  ('core.settings.manage', 'Manage settings', 'core'),
  ('core.flags.manage', 'Manage feature flags', 'core'),
  ('core.license.read', 'Read license entitlements', 'core');

-- System Tenant Admin role (tenant_id NULL = system-defined).
INSERT INTO core.roles (id, tenant_id, name, kind, status)
VALUES ('00000000-0000-4000-8000-000000000001', NULL, 'Tenant Admin', 'system', 'active');

INSERT INTO core.role_permissions (role_id, permission_code)
SELECT '00000000-0000-4000-8000-000000000001', code FROM core.permissions WHERE module_key = 'core';

INSERT INTO core.feature_flags (key, description, default_enabled) VALUES
  ('core.audit.verbose', 'Emit optional AuditEntryAppended bus events', false),
  ('core.dev_header_auth', 'Allow X-Proven-* headers in non-production', true);
