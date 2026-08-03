#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

echo "==> cargo fmt --check"
cargo fmt --all -- --check

echo "==> cargo clippy"
cargo clippy -p proven-api -- -D warnings

echo "==> cargo test"
cargo test --workspace

echo "==> go vet / test"
(cd go && go vet ./... && go test ./...)

echo "==> pnpm typecheck + lint + build"
pnpm install --frozen-lockfile=false
pnpm typecheck
pnpm lint:web
pnpm build:web

echo "CI check passed."
