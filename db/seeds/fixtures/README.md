# Seed fixtures (DX-1)

Offline **demo data** for future UI mocks, contract tests, and (later) SQL seeds.

## Rules

- **No executable business INSERTs** until domain tables exist (platform schema only today).
- JSON fixtures are the source of realistic construction demo content.
- SQL under `../templates/*.sql.template` documents intended row shapes only.

## Domains covered

| File | Purpose |
| --- | --- |
| `companies.json` | Tenant / contractor orgs |
| `projects.json` | Places / jobsites |
| `workers.json` | People on site |
| `equipment.json` | Assets / cranes / vehicles |
| `documents.json` | Controlled docs |
| `flhas.json` | Field level hazard assessments |
| `inspections.json` | Equipment / site inspections |
| `training.json` | Assignments / completions |
| `notifications.json` | Inbox samples |

When Core/Projects/… migrations land, convert templates → versioned `db/seeds/local/*.sql` and keep fixtures for non-SQL consumers.
