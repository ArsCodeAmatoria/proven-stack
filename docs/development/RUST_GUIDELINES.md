# Rust Guidelines

- Workspace members under `crates/` and `apps/`.
- `proven-platform` is the Axum host — no domain rules.
- Prefer `tracing` + structured fields; never log secrets.
- Clippy `-D warnings` in CI and hooks.
- Integration tests in `crates/*/tests/`.

See [`docs/architecture/RUST_BACKEND_ARCHITECTURE.md`](../architecture/RUST_BACKEND_ARCHITECTURE.md).
