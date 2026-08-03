# Commit Conventions

Proven uses [Conventional Commits](https://www.conventionalcommits.org/).

## Format

```text
type(scope)?: subject

body?

footer?
```

### Types

`feat` · `fix` · `docs` · `style` · `refactor` · `perf` · `test` · `build` · `ci` · `chore` · `revert`

### Scopes (common)

`api` · `web` · `workers` · `db` · `ci` · `dx` · `docs` · `docker` · `auth` · `platform` · `deps` · `release`

### Examples

```text
feat(api): add database version endpoint
fix(web): redirect unauthenticated users to login
chore(dx): add lefthook and just recipes
docs(development): document getting started
ci(docker): validate compose configs
```

### Version impact (release-please)

| Commit | SemVer |
| --- | --- |
| `fix:` | patch |
| `feat:` | minor |
| `BREAKING CHANGE:` / `!` | major |

Enforced by commitlint on `commit-msg` (Lefthook).
