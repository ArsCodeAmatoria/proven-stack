#!/usr/bin/env bash
# Scaffold an empty platform migration SQL file.
# Usage: ./scripts/db/new-migration.sh short_description
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

NAME="${1:-}"
if [[ -z "$NAME" ]]; then
  echo "usage: $0 <short_description>"
  exit 1
fi

SAFE="$(echo "$NAME" | tr '[:upper:]' '[:lower:]' | sed -E 's/[^a-z0-9]+/_/g; s/^_|_$//g')"
TS="$(date -u +%Y%m%d%H%M%S)"
DIR="db/migrations/platform"
FILE="${DIR}/${TS}_${SAFE}.sql"

mkdir -p "$DIR"
cat >"$FILE" <<EOF
-- Migration: ${SAFE}
-- Foundation only — no business schema in DX-1.

-- Write expand DDL here.
EOF

echo "Created ${FILE}"
