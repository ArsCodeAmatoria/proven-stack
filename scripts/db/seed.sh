#!/usr/bin/env bash
# Run seed SQL for a profile (local|ci). Foundation seeds are empty placeholders.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

PROFILE="${1:-local}"
shift || true

if [[ ! -f .env ]]; then
  cp .env.example .env
fi

echo "==> proven-migrate seed ${PROFILE}"
cargo run -q -p proven-migrate -- seed "$PROFILE" "$@"
