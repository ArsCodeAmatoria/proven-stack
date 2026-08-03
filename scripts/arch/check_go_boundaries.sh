#!/usr/bin/env bash
# Go workers must remain I/O-only — no domain packages / AuthZ ownership.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT/go"

FAIL=0

# Forbidden import path prefixes (future domain packages).
if go list -f '{{.ImportPath}} {{.Imports}}' ./... 2>/dev/null | grep -E 'proven-stack/go/(domain|internal/domain|pkg/domain)' ; then
  echo "error: Go packages import forbidden domain paths"
  FAIL=1
fi

# Heuristic denylist in source (keep narrow to avoid false positives).
if rg -n --glob '*.go' -e 'package domain\b' -e 'AuthzApi' -e 'business rule' internal cmd 2>/dev/null; then
  echo "error: Go sources matched domain/AuthZ ownership denylist"
  FAIL=1
fi

exit "$FAIL"
