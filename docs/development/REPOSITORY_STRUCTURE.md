# Repository Structure

```text
apps/web          Next.js App Router
apps/api          Rust Axum binary
apps/migrate      SQL migrate/seed CLI
crates/           proven-platform, proven-config, proven-db, proven-observability, proven-shared,
                  proven-events, proven-temporal, modules/*
go/               I/O workers
packages/         shared TS packages
db/               migrations + seeds/fixtures
contracts/        OpenAPI / events (synced)
docs/             PRD, architecture, development handbook, engineering
scripts/          dev, db, ci, arch, codegen
.github/workflows CI + release-please
```

See also [`docs/engineering/GITHUB_REPOSITORY.md`](../engineering/GITHUB_REPOSITORY.md).
