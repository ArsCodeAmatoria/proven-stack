-- Reaffirm projects.* permission codes and grant to Tenant Admin + Project Admin (ADR-0009).

INSERT INTO core.permissions (code, description, module_key) VALUES
  ('projects.project.read', 'Read projects', 'projects'),
  ('projects.project.manage', 'Manage projects', 'projects'),
  ('projects.project.create', 'Create projects', 'projects')
ON CONFLICT (code) DO NOTHING;

-- Tenant Admin
INSERT INTO core.role_permissions (role_id, permission_code)
SELECT '00000000-0000-4000-8000-000000000001', code
FROM core.permissions
WHERE module_key = 'projects'
ON CONFLICT DO NOTHING;

-- Project Admin
INSERT INTO core.role_permissions (role_id, permission_code)
SELECT '00000000-0000-4000-8000-000000000011', code
FROM core.permissions
WHERE module_key = 'projects'
ON CONFLICT DO NOTHING;
