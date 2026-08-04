# Architecture Gates

Automated checks via `just arch` / [`scripts/arch/check.sh`](../../scripts/arch/check.sh):

| Gate | Tool |
| --- | --- |
| Rust crate allowlists | `scripts/arch/check_rust_deps.py` |
| Go I/O-only denylist | `scripts/arch/check_go_boundaries.sh` |
| TS feature isolation | `dependency-cruiser` in `apps/web` |
| Allowed modules under `crates/modules/` | `proven-core`, `proven-companies`, `proven-users`, `proven-projects` |

Rules:

- `proven-platform` may depend on `proven-core`, `proven-companies`, `proven-users`, and `proven-projects`.
- `proven-companies` / `proven-users` / `proven-projects` may depend on `proven-core` (traits) + infra.
- `proven-core` may depend only on infra crates.
- `apps/api` must not path-depend on `crates/modules/*` directly.
- Future feature crates (people, safety, equipment, …) remain forbidden until explicitly added.

Wired into local CI mirror and GitHub Actions **Architecture** job.
