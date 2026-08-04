-- Publish users.* permission codes into Core catalog (ADR-0003 / ADR-0006).

INSERT INTO core.permissions (code, description, module_key) VALUES
  ('users.profile.read', 'Read user account profiles', 'users'),
  ('users.profile.manage', 'Manage user account profiles', 'users'),
  ('users.kind.manage', 'Assign user kind classifications', 'users'),
  ('users.avatar.manage', 'Manage user avatars', 'users'),
  ('users.preferences.manage', 'Manage locale/accessibility/notification prefs', 'users'),
  ('users.auth_profile.manage', 'Manage authentication preference flags', 'users'),
  ('users.signature_profile.manage', 'Manage digital signature profile prefs', 'users'),
  ('users.emergency_contact.manage', 'Manage emergency contacts', 'users'),
  ('users.settings.manage', 'Manage user settings bag', 'users'),
  ('users.audit.read', 'Read user profile audit history', 'users')
ON CONFLICT (code) DO NOTHING;

INSERT INTO core.role_permissions (role_id, permission_code)
SELECT '00000000-0000-4000-8000-000000000001', code
FROM core.permissions
WHERE module_key = 'users'
ON CONFLICT DO NOTHING;
