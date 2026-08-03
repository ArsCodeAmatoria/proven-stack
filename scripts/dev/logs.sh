#!/usr/bin/env bash
# Tail logs from the Proven local Docker development environment.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

if [[ ! -f .env ]]; then
  cp .env.example .env
fi

COMPOSE_FILE="docker/compose/docker-compose.yml"
if [[ "${1:-}" == "--deps-only" ]]; then
  COMPOSE_FILE="docker/compose/docker-compose.deps.yml"
  shift
fi

exec docker compose --env-file .env -f "$COMPOSE_FILE" --project-directory "$ROOT" logs -f "$@"
