-- Seed enterprise permission families + representative system roles (ADR-0007).

INSERT INTO core.permissions (code, description, module_key, family, sensitivity) VALUES
  -- Feature / module gates (evaluated with license+flag preconditions)
  ('feature.module.access', 'Access an entitled module capability', 'feature', 'feature', 'standard'),
  ('feature.flag.evaluate', 'Evaluate feature flags for gated UX', 'feature', 'feature', 'standard'),

  -- Module permissions (catalog published now; modules enforce later)
  ('documents.document.read', 'Read documents', 'documents', 'documents', 'standard'),
  ('documents.document.manage', 'Manage documents', 'documents', 'documents', 'standard'),
  ('documents.version.publish', 'Publish document versions', 'documents', 'documents', 'elevated'),
  ('documents.ack.manage', 'Manage acknowledgements', 'documents', 'documents', 'standard'),
  ('documents.acl.manage', 'Manage document ACL', 'documents', 'documents', 'elevated'),

  ('approvals.request.create', 'Create approval requests', 'approvals', 'approvals', 'standard'),
  ('approvals.request.approve', 'Approve requests', 'approvals', 'approvals', 'elevated'),
  ('approvals.request.reject', 'Reject requests', 'approvals', 'approvals', 'elevated'),
  ('approvals.policy.manage', 'Manage approval policies', 'approvals', 'approvals', 'elevated'),

  ('equipment.asset.read', 'Read equipment assets', 'equipment', 'equipment', 'standard'),
  ('equipment.asset.manage', 'Manage equipment assets', 'equipment', 'equipment', 'standard'),
  ('equipment.inspection.perform', 'Perform inspections', 'equipment', 'equipment', 'standard'),
  ('equipment.readiness.override', 'Override readiness', 'equipment', 'equipment', 'break_glass'),

  ('training.course.read', 'Read training courses', 'training', 'training', 'standard'),
  ('training.course.manage', 'Manage training catalog', 'training', 'training', 'standard'),
  ('training.assignment.manage', 'Manage training assignments', 'training', 'training', 'standard'),
  ('training.completion.record', 'Record training completions', 'training', 'training', 'standard'),

  ('safety.activity.create', 'Create safety activities', 'safety', 'safety', 'standard'),
  ('safety.activity.submit', 'Submit safety activities', 'safety', 'safety', 'standard'),
  ('safety.activity.review', 'Review safety activities', 'safety', 'safety', 'elevated'),
  ('safety.incident.manage', 'Manage incidents', 'safety', 'safety', 'elevated'),
  ('safety.ca.manage', 'Manage corrective actions', 'safety', 'safety', 'elevated'),

  ('projects.project.read', 'Read projects', 'projects', 'projects', 'standard'),
  ('projects.project.manage', 'Manage projects', 'projects', 'projects', 'standard'),
  ('projects.project.create', 'Create projects', 'projects', 'projects', 'standard'),

  ('core.company.read', 'Read companies', 'core', 'core', 'standard'),
  ('core.role.read', 'Read roles', 'core', 'core', 'standard'),
  ('core.grant.read', 'Read grants', 'core', 'core', 'standard'),
  ('core.override.manage', 'Manage permission overrides', 'core', 'core', 'break_glass')
ON CONFLICT (code) DO UPDATE SET
  description = EXCLUDED.description,
  family = EXCLUDED.family,
  sensitivity = EXCLUDED.sensitivity;

-- System roles (fixed UUIDs for deterministic seeds)
INSERT INTO core.roles (id, tenant_id, name, kind, status) VALUES
  ('00000000-0000-4000-8000-000000000010', NULL, 'Company Admin', 'company', 'active'),
  ('00000000-0000-4000-8000-000000000011', NULL, 'Project Admin', 'project', 'active'),
  ('00000000-0000-4000-8000-000000000012', NULL, 'Supervisor', 'project', 'active'),
  ('00000000-0000-4000-8000-000000000013', NULL, 'Worker', 'project', 'active'),
  ('00000000-0000-4000-8000-000000000014', NULL, 'Safety Coordinator', 'project', 'active'),
  ('00000000-0000-4000-8000-000000000015', NULL, 'Equipment Manager', 'company', 'active'),
  ('00000000-0000-4000-8000-000000000016', NULL, 'Training Admin', 'company', 'active'),
  ('00000000-0000-4000-8000-000000000017', NULL, 'Document Control', 'company', 'active'),
  ('00000000-0000-4000-8000-000000000018', NULL, 'Temporary Elevated', 'temporary', 'active')
ON CONFLICT (id) DO NOTHING;

-- Company Admin permissions
INSERT INTO core.role_permissions (role_id, permission_code)
SELECT '00000000-0000-4000-8000-000000000010', code FROM core.permissions
WHERE code IN (
  'core.company.read', 'core.company.manage', 'companies.profile.read', 'companies.profile.manage',
  'users.profile.read', 'feature.module.access'
)
ON CONFLICT DO NOTHING;

-- Project Admin
INSERT INTO core.role_permissions (role_id, permission_code)
SELECT '00000000-0000-4000-8000-000000000011', code FROM core.permissions
WHERE code IN (
  'projects.project.read', 'projects.project.manage', 'core.membership.manage',
  'safety.activity.create', 'safety.activity.review', 'documents.document.read',
  'equipment.asset.read', 'training.assignment.manage', 'feature.module.access'
)
ON CONFLICT DO NOTHING;

-- Supervisor
INSERT INTO core.role_permissions (role_id, permission_code)
SELECT '00000000-0000-4000-8000-000000000012', code FROM core.permissions
WHERE code IN (
  'projects.project.read', 'safety.activity.create', 'safety.activity.review',
  'safety.ca.manage', 'training.completion.record', 'documents.document.read',
  'equipment.asset.read', 'approvals.request.approve'
)
ON CONFLICT DO NOTHING;

-- Worker
INSERT INTO core.role_permissions (role_id, permission_code)
SELECT '00000000-0000-4000-8000-000000000013', code FROM core.permissions
WHERE code IN (
  'projects.project.read', 'safety.activity.create', 'safety.activity.submit',
  'training.course.read', 'documents.document.read', 'equipment.inspection.perform',
  'approvals.request.create'
)
ON CONFLICT DO NOTHING;

-- Safety Coordinator
INSERT INTO core.role_permissions (role_id, permission_code)
SELECT '00000000-0000-4000-8000-000000000014', code FROM core.permissions
WHERE module_key = 'safety' OR code IN ('projects.project.read', 'documents.document.read')
ON CONFLICT DO NOTHING;

-- Equipment Manager
INSERT INTO core.role_permissions (role_id, permission_code)
SELECT '00000000-0000-4000-8000-000000000015', code FROM core.permissions
WHERE module_key = 'equipment' OR code IN ('projects.project.read', 'feature.module.access')
ON CONFLICT DO NOTHING;

-- Training Admin
INSERT INTO core.role_permissions (role_id, permission_code)
SELECT '00000000-0000-4000-8000-000000000016', code FROM core.permissions
WHERE module_key = 'training' OR code IN ('projects.project.read', 'feature.module.access')
ON CONFLICT DO NOTHING;

-- Document Control
INSERT INTO core.role_permissions (role_id, permission_code)
SELECT '00000000-0000-4000-8000-000000000017', code FROM core.permissions
WHERE module_key = 'documents' OR code IN ('approvals.request.approve', 'feature.module.access')
ON CONFLICT DO NOTHING;

-- Temporary Elevated (break-glass capable codes — still needs temporary grant + expiry)
INSERT INTO core.role_permissions (role_id, permission_code)
SELECT '00000000-0000-4000-8000-000000000018', code FROM core.permissions
WHERE sensitivity IN ('elevated', 'break_glass')
ON CONFLICT DO NOTHING;

-- Extend Tenant Admin with override manage + catalog reads
INSERT INTO core.role_permissions (role_id, permission_code)
SELECT '00000000-0000-4000-8000-000000000001', code FROM core.permissions
WHERE code IN ('core.override.manage', 'core.role.read', 'core.grant.read', 'core.company.read', 'feature.module.access')
ON CONFLICT DO NOTHING;
