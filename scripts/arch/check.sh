#!/usr/bin/env bash
# Architecture boundary checks (DX-1). Fail closed on violations.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
FAIL=0

echo "==> arch: Rust crate dependency allowlist"
if ! python3 "$ROOT/scripts/arch/check_rust_deps.py"; then
  FAIL=1
fi

echo "==> arch: Go workers stay I/O-only (denylist scan)"
if ! bash "$ROOT/scripts/arch/check_go_boundaries.sh"; then
  FAIL=1
fi

echo "==> arch: TypeScript feature isolation"
if [[ -f "$ROOT/apps/web/dependency-cruiser.cjs" ]]; then
  if ! pnpm --filter @proven/web exec depcruise --config dependency-cruiser.cjs "app" "components" "features" "lib" 2>/dev/null; then
    # depcruise may not be installed yet in partial checkouts
    if pnpm --filter @proven/web exec -- depcruise -v >/dev/null 2>&1; then
      FAIL=1
    else
      echo "warn: dependency-cruiser not installed in @proven/web — run pnpm install"
    fi
  fi
fi

echo "==> arch: no crates/modules feature crates yet (placeholder guard)"
if [[ -d crates/modules ]] && find crates/modules -name 'Cargo.toml' 2>/dev/null | grep -q .; then
  echo "note: domain crates present — extend check_rust_deps.py module matrix"
fi

if [[ "$FAIL" -ne 0 ]]; then
  echo "Architecture checks FAILED"
  exit 1
fi
echo "Architecture checks passed."
