# Release Process

- Conventional Commits drive versions via **release-please**.
- On push to `main`, [`.github/workflows/release-please.yml`](../../.github/workflows/release-please.yml) opens/updates a release PR.
- Merging the release PR tags, updates [`CHANGELOG.md`](../../CHANGELOG.md), and creates a GitHub Release.
- No package registry publish in DX-1 (no deployment sprint).
