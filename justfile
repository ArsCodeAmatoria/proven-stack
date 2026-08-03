# Proven developer task runner (source of truth).
# Install: https://github.com/casey/just  (brew install just)
# Makefile targets wrap these recipes for compatibility.

set shell := ["bash", "-euo", "pipefail", "-c"]

root := justfile_directory()

default:
    @just --list

# --- Onboarding ----------------------------------------------------------------

# One-command developer setup (prereqs, deps, docker, migrate, seed).
setup *ARGS:
    {{root}}/scripts/dev/setup.sh {{ARGS}}

# Install Lefthook git hooks.
hooks:
    pnpm exec lefthook install
    @echo "Lefthook hooks installed."

# --- Docker --------------------------------------------------------------------

up:
    {{root}}/scripts/dev/up.sh

down:
    {{root}}/scripts/dev/down.sh

logs:
    {{root}}/scripts/dev/logs.sh

ps:
    {{root}}/scripts/dev/ps.sh

deps:
    {{root}}/scripts/dev/up.sh --deps-only

# --- Apps ----------------------------------------------------------------------

api:
    cargo run -p proven-api

web:
    pnpm --filter @proven/web dev

worker name="notify":
    cd go && go run ./cmd/{{name}}-worker

# Concurrent api + web + notify-worker (Ctrl+C stops all).
dev:
    {{root}}/scripts/dev/dev.sh

# --- Quality -------------------------------------------------------------------

fmt:
    cargo fmt --all
    cd go && gofmt -w .
    pnpm exec prettier --write "apps/**/*.{ts,tsx,js,jsx,json,css,md}" "packages/**/*.{ts,tsx,js,jsx,json,md}" "*.md" ".github/**/*.{yml,yaml}" || true

lint:
    cargo clippy --workspace --all-targets -- -D warnings
    cd go && test -z "$(gofmt -l .)" || (echo "gofmt needed" && gofmt -l . && exit 1)
    cd go && go vet ./...
    pnpm lint:web

check: lint
    pnpm typecheck
    cargo test --workspace --quiet
    cd go && go test ./...

ci:
    {{root}}/scripts/ci/check.sh

arch:
    {{root}}/scripts/arch/check.sh

# --- Test ----------------------------------------------------------------------

test-fast:
    cargo test -p proven-shared -p proven-config -p proven-platform --quiet
    cd go && go test ./...
    pnpm typecheck

test:
    cargo test --workspace
    cd go && go test ./...
    pnpm typecheck
    pnpm --filter @proven/web test:unit

test-e2e:
    pnpm --filter @proven/web test:e2e

test-coverage:
    {{root}}/scripts/ci/coverage.sh

# --- Build ---------------------------------------------------------------------

build: build-api build-workers build-web

build-api:
    cargo build -p proven-api -p proven-migrate

build-web:
    pnpm build:web

build-workers:
    cd go && go build ./...

# --- Database ------------------------------------------------------------------

db-migrate:
    {{root}}/scripts/db/migrate.sh

db-seed profile="local":
    {{root}}/scripts/db/seed.sh {{profile}}

db-reset:
    {{root}}/scripts/db/reset.sh

db-migrate-create name:
    {{root}}/scripts/db/new-migration.sh {{name}}

# --- Docs ----------------------------------------------------------------------

docs:
    {{root}}/scripts/codegen/export-openapi.sh
    @echo "Handbook: docs/development/README.md"
    @echo "Swagger:  http://127.0.0.1:8080/docs"
    @echo "Redoc:    http://127.0.0.1:8080/redoc"
