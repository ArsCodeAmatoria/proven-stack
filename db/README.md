# Database

PostgreSQL for Proven: migrations, seeds, and pool configuration.

## Layout

```text
db/
├── migrations/
│   ├── platform/          # sqlx ledger + platform schema (outbox, …)
│   ├── core/              # Core foundation schema
│   ├── companies/         # Company profile schema (ADR-0005)
│   └── users/             # User account profile schema (ADR-0006)
├── seeds/
│   ├── local/
│   ├── ci/
│   ├── fixtures/          # offline demo JSON (not executed)
│   └── templates/         # future INSERT shapes (comment-only)
└── README.md
```

## Commands

```bash
# Apply platform then core migrations
./scripts/db/migrate.sh
# or
cargo run -p proven-migrate -- migrate
```

## Rules

- Schema ownership: `platform`, `core`, `companies`, `users`. No Projects/People schemas yet.
- No cross-schema FKs; UUID references only ([ADR-0004](../docs/adr/0004-core-persistence.md), [ADR-0005](../docs/adr/0005-companies-profile-module.md), [ADR-0006](../docs/adr/0006-users-profile-module.md)).
- Other modules must not SQL into these schemas — use public APIs.
