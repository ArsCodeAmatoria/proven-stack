#!/usr/bin/env bash
# Sync contracts/openapi/openapi.json from the utoipa ApiDoc.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

echo "==> exporting OpenAPI via proven-platform test"
RUN_OPENAPI_EXPORT=1 cargo test -p proven-platform --test openapi_export -- --nocapture

if [[ -f contracts/openapi/openapi.json ]]; then
  echo "Wrote contracts/openapi/openapi.json"
else
  echo "error: openapi.json was not written"
  exit 1
fi
