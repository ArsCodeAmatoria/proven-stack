# Testing Guide

| Layer | Location | Runner |
| --- | --- | --- |
| Rust unit/integration | `crates/**`, `crates/*/tests` | `cargo test` |
| Go unit | `go/**/*_test.go` | `go test ./...` |
| Web unit | `apps/web/tests/unit` | `pnpm --filter @proven/web test:unit` |
| Web e2e | `apps/web/tests/e2e` | `pnpm --filter @proven/web test:e2e` |
| Cross-app e2e / load | `tests/e2e`, `tests/load` | later |

```bash
just test-fast    # pre-push
just test         # broader
just test-e2e     # Playwright
just test-coverage
```

Coverage targets (aspirational; not hard-gated in DX-1): raise shared-lib floors as modules land. Reports via Codecov when `CODECOV_TOKEN` is set.

Badge (after Codecov is linked): `![coverage](https://codecov.io/gh/<org>/<repo>/branch/main/graph/badge.svg)`

See [`docs/architecture/TESTING_STRATEGY.md`](../architecture/TESTING_STRATEGY.md) and [Engineering Metrics](./ENGINEERING_METRICS.md).
