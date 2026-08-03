#!/usr/bin/env bash
# Apply platform PostgreSQL migrations (sqlx metadata + platform schema only).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

if [[ ! -f .env ]]; then
  cp .env.example .env
fi

echo "==> proven-migrate migrate"
cargo run -q -p proven-migrate -- migrate "$@"
