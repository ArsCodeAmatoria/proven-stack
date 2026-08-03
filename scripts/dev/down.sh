#!/usr/bin/env bash
# Stop the Proven local Docker development environment.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

VOLUMES=0
DEPS_ONLY=0

usage() {
  cat <<'EOF'
Usage: scripts/dev/down.sh [options]

Stop Compose services started for local development.

Options:
  --deps-only   Target the deps-only compose file
  -v, --volumes Also remove named volumes (destructive: DB/Redis data)
  -h, --help    Show this help
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --deps-only) DEPS_ONLY=1; shift ;;
    -v|--volumes) VOLUMES=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown option: $1" >&2; usage; exit 1 ;;
  esac
done

COMPOSE_FILE="docker/compose/docker-compose.yml"
if [[ "$DEPS_ONLY" -eq 1 ]]; then
  COMPOSE_FILE="docker/compose/docker-compose.deps.yml"
fi

ARGS=(compose --env-file .env -f "$COMPOSE_FILE" --project-directory "$ROOT" down)
if [[ "$VOLUMES" -eq 1 ]]; then
  ARGS+=(--volumes)
  echo "==> Stopping stack and removing volumes ($COMPOSE_FILE)"
else
  echo "==> Stopping stack ($COMPOSE_FILE)"
fi

# Ensure .env exists so --env-file does not fail when never bootstrapped.
if [[ ! -f .env ]]; then
  cp .env.example .env
fi

docker "${ARGS[@]}"

# Always tear down the sibling project name if both were used.
OTHER="docker/compose/docker-compose.deps.yml"
if [[ "$DEPS_ONLY" -eq 1 ]]; then
  OTHER="docker/compose/docker-compose.yml"
fi
if [[ -f .env ]]; then
  EXTRA=(compose --env-file .env -f "$OTHER" --project-directory "$ROOT" down)
  if [[ "$VOLUMES" -eq 1 ]]; then
    EXTRA+=(--volumes)
  fi
  docker "${EXTRA[@]}" >/dev/null 2>&1 || true
fi

echo "Stopped."
