# Proven — CI & Branch Protection

| Field | Value |
| --- | --- |
| **Document type** | Engineering / DevEx |
| **Status** | Active |
| **Last updated** | 2026-08-03 |
| **Companion** | [GitHub Repository Design](./GITHUB_REPOSITORY.md), [Development](./DEVELOPMENT.md) |

---

## 1. Workflows (no deployment)

| Workflow | Role |
| --- | --- |
| [`ci.yml`](../../.github/workflows/ci.yml) | PR/main orchestrator + **PR Validation** gate |
| [`ci-rust.yml`](../../.github/workflows/ci-rust.yml) | fmt, clippy, test, optional coverage → Codecov |
| [`ci-go.yml`](../../.github/workflows/ci-go.yml) | gofmt, vet, test, optional coverage → Codecov |
| [`ci-web.yml`](../../.github/workflows/ci-web.yml) | typecheck, lint, Vitest, arch, build + optional Codecov |
| [`ci-docker.yml`](../../.github/workflows/ci-docker.yml) | Compose validate + image builds (no push) + digests |
| [`release-please.yml`](../../.github/workflows/release-please.yml) | Conventional Commits → CHANGELOG / GitHub Release |
| Architecture job in [`ci.yml`](../../.github/workflows/ci.yml) | `scripts/arch/check.sh` boundary gates |
| [`labeler.yml`](../../.github/workflows/labeler.yml) | Path → PR labels |

Deploy workflows are intentionally **not** present yet.

Coverage uploads run when `CODECOV_TOKEN` is set (skipped otherwise). Status checks are **informational** in DX-1 — see [`codecov.yml`](../../codecov.yml) and [Engineering Metrics](../development/ENGINEERING_METRICS.md).

### Pipeline coverage

| Stage | Rust | Go | Next.js | Docker |
| --- | --- | --- | --- | --- |
| Lint | `fmt` + `clippy -D warnings` | `gofmt` + `vet` | ESLint | Compose `config` |
| Build | `proven-api`, `proven-migrate` | all `cmd/*` | `pnpm build:web` | `Dockerfile.api` + workers |
| Test | `cargo test --workspace` | `go test ./...` | Vitest unit + typecheck | image build smoke |
| Coverage | llvm-cov (optional upload) | go cover (optional) | Vitest c8/v8 (optional) | — |
| Arch | crate allowlist | I/O denylist | dependency-cruiser | — |
| Artifacts | debug binaries (7d) | worker binaries (7d) | `BUILD_ID` + revision | image IDs |

Path filters skip irrelevant jobs; the **PR Validation** job always runs and fails if any executed job failed.

### Local mirror

```bash
just ci
# or
./scripts/ci/check.sh
```

---

## 2. Branch protection recommendations (`main`)

Configure in GitHub → **Settings → Branches → Branch protection rules** (or Rulesets).

### Required

| Setting | Value |
| --- | --- |
| Restrict who can push | On (no direct pushes; PRs only) |
| Require a pull request before merging | On |
| Required approving reviews | ≥ 1 (2 for security-sensitive paths via CODEOWNERS) |
| Require review from CODEOWNERS | On |
| Require status checks to pass | On |
| Required check | **`PR Validation`** (job name from `ci.yml`) |
| Require branches to be up to date | On (recommended) |
| Require conversation resolution | On |
| Do not allow bypassing the above settings | On (admins included) |
| Allow force pushes | **Off** |
| Allow deletions | **Off** |

### Recommended

| Setting | Value |
| --- | --- |
| Require linear history | On (matches squash-merge preference) |
| Squash merge | Default; disable merge commits if desired |
| Lock branch | Off (unless freezing a release cut) |

### Ruleset JSON sketch (GitHub Rulesets API)

Require check context exactly:

```text
PR Validation
```

Do **not** require path-filtered job names (`Rust` / `Go` / …) as standalone required checks — skipped jobs block merges when listed as required.

### Apply via `gh` (example)

```bash
# Requires admin on the repo. Adjust owner/repo.
gh api \
  --method PUT \
  -H "Accept: application/vnd.github+json" \
  "/repos/ArsCodeAmatoria/proven-stack/branches/main/protection" \
  -f required_status_checks='{"strict":true,"contexts":["PR Validation"]}' \
  -F enforce_admins=true \
  -f required_pull_request_reviews='{"required_approving_review_count":1,"require_code_owner_reviews":true}' \
  -F allow_force_pushes=false \
  -F allow_deletions=false \
  -F restrictions=null
```

Prefer **Repository Rulesets** in the UI for newer orgs; keep the same required check name.

---

## 3. PR validation expectations

- Green **PR Validation** before merge.  
- CODEOWNERS approvals for owned paths.  
- No secrets in the diff.  
- No production deploy from CI (staging/prod workflows come later).  

---

## 4. Out of scope (later)

- Deploy to Fly / Vercel / Cloudflare  
- Container registry push  
- CodeQL / Gitleaks (optional hardening)  
- Playwright e2e nightly  
