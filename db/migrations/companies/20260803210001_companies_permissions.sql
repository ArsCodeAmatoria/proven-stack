-- Publish companies.* permission codes into Core catalog (ADR-0003 / ADR-0005).

INSERT INTO core.permissions (code, description, module_key) VALUES
  ('companies.profile.read', 'Read company profile', 'companies'),
  ('companies.profile.manage', 'Manage company profile shell', 'companies'),
  ('companies.unit.manage', 'Manage business units', 'companies'),
  ('companies.address.manage', 'Manage company addresses', 'companies'),
  ('companies.contact.manage', 'Manage company contacts', 'companies'),
  ('companies.branding.manage', 'Manage company branding', 'companies'),
  ('companies.safety_settings.manage', 'Manage company safety settings', 'companies'),
  ('companies.regional_settings.manage', 'Manage company regional settings', 'companies'),
  ('companies.templates.manage', 'Manage company default template pointers', 'companies'),
  ('companies.notification_defaults.manage', 'Manage company notification defaults', 'companies'),
  ('companies.storage.manage', 'Manage company storage configuration', 'companies')
ON CONFLICT (code) DO NOTHING;

-- Grant companies permissions to system Tenant Admin role.
INSERT INTO core.role_permissions (role_id, permission_code)
SELECT '00000000-0000-4000-8000-000000000001', code
FROM core.permissions
WHERE module_key = 'companies'
ON CONFLICT DO NOTHING;
