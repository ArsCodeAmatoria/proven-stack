# Security Practices

- Never commit secrets; gitleaks runs on pre-commit when installed.
- Dependabot weekly updates ([`.github/dependabot.yml`](../../.github/dependabot.yml)); enable Dependabot security alerts in GitHub settings.
- Prefer `BETTER_AUTH_SECRET` / `PROVEN_SESSION_SECRET` ≥ 32 chars outside local defaults.
- Health endpoints stay unauthenticated but non-revealing.
- See [`SECURITY.md`](../../SECURITY.md) and architecture security docs.
