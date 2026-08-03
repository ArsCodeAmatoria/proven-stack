.PHONY: help bootstrap install deps-up deps-down build build-api build-web build-workers \
	dev-api dev-web dev-worker-notify test lint fmt check

help:
	@echo "Proven foundation targets:"
	@echo "  make bootstrap     - install JS deps + print next steps"
	@echo "  make install       - pnpm install"
	@echo "  make deps-up       - start docker compose core deps"
	@echo "  make deps-down     - stop docker compose"
	@echo "  make build         - build api, web, workers"
	@echo "  make build-api     - cargo build -p proven-api"
	@echo "  make build-web     - pnpm build:web"
	@echo "  make build-workers - go build ./..."
	@echo "  make dev-api       - run proven-api"
	@echo "  make dev-web       - run Next.js"
	@echo "  make dev-worker-notify - run notify-worker"
	@echo "  make check         - fmt/clippy + go vet + web typecheck"

bootstrap: install
	@cp -n .env.example .env 2>/dev/null || true
	@echo "Bootstrap complete. Start API: make dev-api | Web: make dev-web | Deps: make deps-up"

install:
	pnpm install

deps-up:
	docker compose -f docker/compose/docker-compose.yml up -d

deps-down:
	docker compose -f docker/compose/docker-compose.yml down

build: build-api build-workers build-web

build-api:
	cargo build -p proven-api

build-web:
	pnpm build:web

build-workers:
	cd go && go build ./...

dev-api:
	cargo run -p proven-api

dev-web:
	pnpm dev:web

dev-worker-notify:
	cd go && go run ./cmd/notify-worker

fmt:
	cargo fmt --all
	cd go && gofmt -w .

lint:
	cargo clippy -p proven-api -- -D warnings
	cd go && go vet ./...
	pnpm lint:web

check: lint
	pnpm typecheck
	cargo test -p proven-shared --quiet
	cd go && go test ./...

test:
	cargo test --workspace
	cd go && go test ./...
