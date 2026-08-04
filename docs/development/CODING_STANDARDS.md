# Coding Standards

- Prefer clarity and maintainability over cleverness.
- No business rules in React or Go workers — Rust domain modules own authority ([`AGENTS.md`](../../AGENTS.md)).
- New HTTP endpoints follow [REST API Conventions](./REST_API_CONVENTIONS.md) (envelopes, cursor lists, nested errors, `/api/v1`).
- Format with language-native tools + Prettier for TS/JSON/MD.
- Lint must be clean (`-D warnings` for Clippy; ESLint max-warnings 0 on staged files).
- Secrets never in git; use `.env` (gitignored).
- Conventional Commits required ([Commit Conventions](./COMMIT_CONVENTIONS.md)).
