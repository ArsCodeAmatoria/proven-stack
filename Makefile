.PHONY: help bootstrap install deps-up deps-down \
	docker-up docker-down docker-logs docker-ps docker-deps docker-build \
	build build-api build-web build-workers \
	dev-api dev-web dev-worker-notify test lint fmt check ci setup hooks arch

# Thin wrappers around `just` (source of truth: justfile).
# Falls back to legacy recipes if `just` is not installed.

HAS_JUST := $(shell command -v just 2>/dev/null)

help:
ifeq ($(HAS_JUST),)
	@echo "Install 'just' (brew install just) for the full recipe list."
	@echo "Legacy make targets still work below."
else
	@just --list
endif

bootstrap: setup

setup:
ifdef HAS_JUST
	just setup
else
	./scripts/dev/setup.sh
endif

hooks:
ifdef HAS_JUST
	just hooks
else
	pnpm exec lefthook install
endif

install:
	pnpm install

docker-up:
ifdef HAS_JUST
	just up
else
	./scripts/dev/up.sh
endif

docker-down:
ifdef HAS_JUST
	just down
else
	./scripts/dev/down.sh
endif

docker-deps:
ifdef HAS_JUST
	just deps
else
	./scripts/dev/up.sh --deps-only
endif

docker-logs:
ifdef HAS_JUST
	just logs
else
	./scripts/dev/logs.sh
endif

docker-ps:
ifdef HAS_JUST
	just ps
else
	./scripts/dev/ps.sh
endif

docker-build:
	./scripts/dev/up.sh --build

deps-up: docker-deps

deps-down: docker-down

build:
ifdef HAS_JUST
	just build
else
	$(MAKE) build-api build-workers build-web
endif

build-api:
	cargo build -p proven-api -p proven-migrate

build-web:
	pnpm build:web

build-workers:
	cd go && go build ./...

dev-api:
ifdef HAS_JUST
	just api
else
	cargo run -p proven-api
endif

dev-web:
ifdef HAS_JUST
	just web
else
	pnpm --filter @proven/web dev
endif

dev-worker-notify:
ifdef HAS_JUST
	just worker notify
else
	cd go && go run ./cmd/notify-worker
endif

fmt:
ifdef HAS_JUST
	just fmt
else
	cargo fmt --all
	cd go && gofmt -w .
endif

lint:
ifdef HAS_JUST
	just lint
else
	cargo clippy --workspace --all-targets -- -D warnings
	cd go && go vet ./...
	pnpm lint:web
endif

check:
ifdef HAS_JUST
	just check
else
	$(MAKE) lint
	pnpm typecheck
	cargo test --workspace --quiet
	cd go && go test ./...
endif

ci:
ifdef HAS_JUST
	just ci
else
	./scripts/ci/check.sh
endif

arch:
ifdef HAS_JUST
	just arch
else
	./scripts/arch/check.sh
endif

test:
ifdef HAS_JUST
	just test
else
	cargo test --workspace
	cd go && go test ./...
endif
