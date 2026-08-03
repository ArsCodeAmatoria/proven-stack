# Pull Request Process

1. Branch from `main`; keep PRs focused.
2. Use the PR template; include test plan.
3. Ensure **PR Validation** is green.
4. Request CODEOWNERS review when paths require it.
5. Prefer squash merge.

Local preflight:

```bash
just test-fast
just arch
./scripts/ci/check.sh   # fuller mirror
```

See [`CONTRIBUTING.md`](../../CONTRIBUTING.md).
