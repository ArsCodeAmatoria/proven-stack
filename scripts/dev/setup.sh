#!/usr/bin/env bash
# One-command developer setup for Proven (DX-1).
# Usage: ./scripts/dev/setup.sh [--deps-only] [--skip-docker]
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

DEPS_ONLY=false
SKIP_DOCKER=false
for arg in "$@"; do
  case "$arg" in
    --deps-only) DEPS_ONLY=true ;;
    --skip-docker) SKIP_DOCKER=true ;;
    -h|--help)
      echo "Usage: $0 [--deps-only] [--skip-docker]"
      exit 0
      ;;
  esac
done

need() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "error: missing prerequisite: $1"
    exit 1
  fi
}

echo "==> validating prerequisites"
need rustc
need cargo
need go
need node
need pnpm
if [[ "$SKIP_DOCKER" != "true" ]]; then
  need docker
fi

if ! command -v just >/dev/null 2>&1; then
  echo "warn: 'just' not found — install from https://github.com/casey/just (brew install just)"
fi

NODE_MAJOR="$(node -p "process.versions.node.split('.')[0]")"
NODE_MINOR="$(node -p "process.versions.node.split('.')[1]")"
if [[ "$NODE_MAJOR" -lt 20 ]] || { [[ "$NODE_MAJOR" -eq 20 ]] && [[ "$NODE_MINOR" -lt 19 ]]; }; then
  echo "warn: Node >= 20.19 recommended (found $(node -v))"
fi

if [[ ! -f .env ]]; then
  cp .env.example .env
  echo "Created .env from .env.example"
fi

echo "==> installing JS dependencies"
pnpm install --frozen-lockfile=false

echo "==> fetching Rust / Go deps"
cargo fetch
(cd go && go mod download)

echo "==> building foundation binaries"
cargo build -p proven-api -p proven-migrate
(cd go && go build ./...)

if command -v lefthook >/dev/null 2>&1 || [[ -x node_modules/.bin/lefthook ]]; then
  echo "==> installing git hooks"
  pnpm exec lefthook install || true
else
  echo "warn: lefthook unavailable — run 'pnpm install' then 'just hooks'"
fi

if [[ "$SKIP_DOCKER" == "true" ]]; then
  echo "Skipping Docker (--skip-docker)."
  echo "Setup complete (deps only)."
  exit 0
fi

if [[ "$DEPS_ONLY" == "true" ]]; then
  echo "==> starting infra containers"
  ./scripts/dev/up.sh --deps-only
else
  echo "==> starting full Docker stack"
  ./scripts/dev/up.sh
fi

echo "==> waiting for Postgres"
for i in $(seq 1 60); do
  if docker compose -f docker/compose/docker-compose.yml --project-directory "$ROOT" \
    exec -T postgres pg_isready -U proven -d proven >/dev/null 2>&1 \
    || docker compose -f docker/compose/docker-compose.deps.yml --project-directory "$ROOT" \
      exec -T postgres pg_isready -U proven -d proven >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

echo "==> migrations + seed"
./scripts/db/migrate.sh || echo "warn: migrate skipped/failed (is Postgres up?)"
./scripts/db/seed.sh local || echo "warn: seed skipped/failed"

cat <<EOF

Setup complete.

Next:
  just api              # Rust API  :8080
  just web              # Next.js   :3000
  just worker notify    # Go worker :8091
  just dev              # all three together

Handbook: docs/development/GETTING_STARTED.md
EOF
