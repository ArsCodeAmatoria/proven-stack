# Branch Strategy

Trunk-based development on `main`.

| Pattern | Use |
| --- | --- |
| `feat/…` | Features |
| `fix/…` | Bug fixes |
| `chore/…` | Tooling / DX |
| `docs/…` | Documentation |
| `hotfix/…` | Urgent production fixes (when deployed) |

- Short-lived branches → PR → squash merge.
- No direct commits to `main`.
- Required check: **PR Validation** ([CI handbook](./CI_CD.md)).
