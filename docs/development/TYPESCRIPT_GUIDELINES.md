# TypeScript Guidelines

- App Router under `apps/web`; shared libs in `packages/`.
- Prefer feature folders (`features/*`) for screens; keep pages thin.
- No business AuthZ in the client — UX hints only.
- Prettier + ESLint; Vitest for unit; Playwright for e2e smoke.

See [`docs/architecture/FRONTEND_FOLDER_STRUCTURE.md`](../architecture/FRONTEND_FOLDER_STRUCTURE.md).
