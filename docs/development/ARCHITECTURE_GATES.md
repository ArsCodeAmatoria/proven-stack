# Architecture Gates

Automated checks via `just arch` / [`scripts/arch/check.sh`](../../scripts/arch/check.sh):

| Gate | Tool |
| --- | --- |
| Rust crate allowlists | `scripts/arch/check_rust_deps.py` |
| Go I/O-only denylist | `scripts/arch/check_go_boundaries.sh` |
| TS feature isolation | `dependency-cruiser` in `apps/web` |

Wired into local CI mirror and GitHub Actions **Architecture** job.

When domain crates appear, extend the Rust matrix so feature crates cannot import each other.
