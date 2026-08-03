#!/usr/bin/env bash
# Start the Proven local Docker development environment.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

COMPOSE_FILE="docker/compose/docker-compose.yml"
DEPS_ONLY=0
BUILD=0
DETACH=1
EXTRA_ARGS=()

usage() {
  cat <<'EOF'
Usage: scripts/dev/up.sh [options] [-- compose-args...]

Start the Proven Docker Compose development stack.

Options:
  --deps-only   Start infrastructure only (Postgres, Redis, NATS, Temporal, UI)
  --build       Force image rebuild
  --foreground  Attach to logs (do not detach)
  -h, --help    Show this help

Examples:
  ./scripts/dev/up.sh
  ./scripts/dev/up.sh --deps-only
  ./scripts/dev/up.sh --build --foreground
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --deps-only) DEPS_ONLY=1; shift ;;
    --build) BUILD=1; shift ;;
    --foreground) DETACH=0; shift ;;
    -h|--help) usage; exit 0 ;;
    --) shift; EXTRA_ARGS+=("$@"); break ;;
    *) EXTRA_ARGS+=("$1"); shift ;;
  esac
done

if [[ ! -f .env ]]; then
  cp .env.example .env
  echo "Created .env from .env.example"
fi

if [[ "$DEPS_ONLY" -eq 1 ]]; then
  COMPOSE_FILE="docker/compose/docker-compose.deps.yml"
fi

ARGS=(compose --env-file .env -f "$COMPOSE_FILE" --project-directory "$ROOT")

if [[ "$BUILD" -eq 1 ]]; then
  echo "==> Building images"
  docker "${ARGS[@]}" build "${EXTRA_ARGS[@]}"
fi

UP_ARGS=(up)
if [[ "$DETACH" -eq 1 ]]; then
  UP_ARGS+=(-d)
fi
if [[ "$BUILD" -eq 1 ]]; then
  UP_ARGS+=(--build)
fi

echo "==> Starting stack ($COMPOSE_FILE)"
docker "${ARGS[@]}" "${UP_ARGS[@]}" "${EXTRA_ARGS[@]}"

if [[ "$DETACH" -eq 1 ]]; then
  echo
  echo "Stack is up. Useful URLs:"
  echo "  Web            http://localhost:3000"
  echo "  API            http://localhost:8080/healthz"
  echo "  Worker         http://localhost:8091/healthz"
  echo "  Temporal UI    http://localhost:8088"
  echo "  NATS monitor   http://localhost:8222"
  echo "  Postgres       localhost:5432  (proven/proven)"
  echo "  Redis          localhost:6379"
  echo "  Temporal gRPC  localhost:7233"
  echo
  echo "Logs:  ./scripts/dev/logs.sh"
  echo "Stop:  ./scripts/dev/down.sh"
  if [[ "$DEPS_ONLY" -eq 1 ]]; then
    echo "(deps-only: app containers were not started)"
  fi
fi
