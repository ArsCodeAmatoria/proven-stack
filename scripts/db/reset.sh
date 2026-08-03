#!/usr/bin/env bash
# Drop and recreate the local development database, then migrate + seed.
# Requires Docker Postgres from compose (user/db: proven).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

COMPOSE=(docker compose -f docker/compose/docker-compose.yml --project-directory "$ROOT")
if ! "${COMPOSE[@]}" ps postgres --status running >/dev/null 2>&1; then
  COMPOSE=(docker compose -f docker/compose/docker-compose.deps.yml --project-directory "$ROOT")
fi

echo "==> resetting database proven"
"${COMPOSE[@]}" exec -T postgres psql -U proven -d postgres -v ON_ERROR_STOP=1 <<'SQL'
SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = 'proven' AND pid <> pg_backend_pid();
DROP DATABASE IF EXISTS proven;
CREATE DATABASE proven OWNER proven;
SQL

echo "==> migrate"
./scripts/db/migrate.sh

echo "==> seed local"
./scripts/db/seed.sh local

echo "Database reset complete."
