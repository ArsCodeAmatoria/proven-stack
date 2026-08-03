#!/usr/bin/env bash
# Generate language coverage reports (DX-1 baseline; CI uploads when CODECOV_TOKEN set).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
mkdir -p coverage

echo "==> Go coverage"
(cd go && go test ./... -coverprofile="$ROOT/coverage/go.out" -covermode=atomic)
go tool cover -func=coverage/go.out | tail -n 1 || true

echo "==> Web Vitest coverage"
pnpm --filter @proven/web test:unit -- --coverage || true

if command -v cargo-llvm-cov >/dev/null 2>&1; then
  echo "==> Rust llvm-cov"
  cargo llvm-cov --workspace --lcov --output-path coverage/rust.lcov || true
else
  echo "warn: cargo-llvm-cov not installed (cargo install cargo-llvm-cov)"
fi

echo "Coverage artifacts under ./coverage (and apps/web/coverage)"
