#!/usr/bin/env bash
# Apply platform → core → companies PostgreSQL migrations (shared _sqlx_migrations ledger).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

if [[ ! -f .env ]]; then
  cp .env.example .env
fi

echo "==> proven-migrate migrate (platform)"
cargo run -q -p proven-migrate -- migrate --dir db/migrations/platform

echo "==> proven-migrate migrate (core)"
cargo run -q -p proven-migrate -- migrate --dir db/migrations/core

echo "==> proven-migrate migrate (companies)"
cargo run -q -p proven-migrate -- migrate --dir db/migrations/companies

echo "==> proven-migrate migrate (users)"
cargo run -q -p proven-migrate -- migrate --dir db/migrations/users

echo "==> proven-migrate migrate (projects)"
cargo run -q -p proven-migrate -- migrate --dir db/migrations/projects
