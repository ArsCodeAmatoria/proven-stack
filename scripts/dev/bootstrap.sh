#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

if [[ ! -f .env ]]; then
  cp .env.example .env
  echo "Created .env from .env.example"
fi

echo "==> pnpm install"
pnpm install

echo "==> cargo build -p proven-api"
cargo build -p proven-api

echo "==> go build"
(cd go && go build ./...)

echo "Foundation bootstrap OK."
echo "  make docker-up            # full Docker stack (recommended)"
echo "  make docker-deps          # infra only"
echo "  make dev-api              # http://127.0.0.1:8080/healthz"
echo "  make dev-web              # http://127.0.0.1:3000"
echo "  make dev-worker-notify    # http://127.0.0.1:8091/healthz"
