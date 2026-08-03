#!/usr/bin/env bash
# Run API + web + notify-worker together with clean shutdown.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

PIDS=()
cleanup() {
  echo ""
  echo "==> stopping processes"
  for pid in "${PIDS[@]:-}"; do
    kill "$pid" 2>/dev/null || true
  done
  wait 2>/dev/null || true
}
trap cleanup EXIT INT TERM

echo "==> starting proven-api"
cargo run -p proven-api &
PIDS+=($!)

echo "==> starting web"
pnpm --filter @proven/web dev &
PIDS+=($!)

echo "==> starting notify-worker"
(cd go && PROVEN_INFRA_OPTIONAL="${PROVEN_INFRA_OPTIONAL:-true}" go run ./cmd/notify-worker) &
PIDS+=($!)

echo "API :8080 · Web :3000 · Worker :8091  (Ctrl+C to stop)"
wait
