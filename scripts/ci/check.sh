#!/usr/bin/env bash
# Local mirror of GitHub Actions CI (no Docker image builds).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

echo "==> architecture gates"
./scripts/arch/check.sh

echo "==> cargo fmt --check"
cargo fmt --all -- --check

echo "==> cargo clippy (workspace)"
cargo clippy --workspace --all-targets -- -D warnings

echo "==> cargo test"
cargo test --workspace

echo "==> cargo build"
cargo build -p proven-api -p proven-migrate

echo "==> gofmt"
(
  cd go
  unformatted="$(gofmt -l .)"
  if [[ -n "$unformatted" ]]; then
    echo "gofmt needed on:"
    echo "$unformatted"
    exit 1
  fi
)

echo "==> go vet / test / build"
(
  cd go
  go vet ./...
  go test ./...
  go build ./...
)

echo "==> pnpm typecheck + lint + unit + build"
pnpm install --frozen-lockfile
pnpm typecheck
pnpm lint:web
pnpm --filter @proven/web test:unit
NEXT_PUBLIC_PROVEN_API_URL="${NEXT_PUBLIC_PROVEN_API_URL:-http://127.0.0.1:8080}" \
  PROVEN_API_URL="${PROVEN_API_URL:-http://127.0.0.1:8080}" \
  pnpm build:web

echo "CI check passed."
