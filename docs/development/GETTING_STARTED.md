# Getting Started

## Prerequisites

| Tool | Version |
| --- | --- |
| Docker Desktop / Engine | recent |
| Rust | `1.86.0` ([`rust-toolchain.toml`](../../rust-toolchain.toml)) |
| Go | `1.22+` |
| Node.js | `≥ 20.19` |
| pnpm | `9.15` |
| just | recommended (`brew install just`) |
| gitleaks | recommended for secret hooks |

## One command

```bash
git clone <repo-url> proven-stack
cd proven-stack
./scripts/dev/setup.sh
# or: just setup
```

Flags:

- `--deps-only` — infra containers only (no app containers)
- `--skip-docker` — install deps only

Then:

```bash
just api    # :8080
just web    # :3000
just worker notify
# or
just dev    # all three
```

Handbook continues in [Local Development](./LOCAL_DEVELOPMENT.md).

Install git hooks after clone (also run by `just setup`):

```bash
just hooks
```

Pre-commit runs fmt/lint/secrets; pre-push runs `just test-fast`. Full suite stays in CI.
