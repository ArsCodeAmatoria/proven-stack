# @proven/web

Next.js App Router foundation shell for Proven.

## Routes

| Path | Layout | Purpose |
| --- | --- | --- |
| `/` | — | Dashboard if signed in; login otherwise |
| `/login` | Auth (guest) | Better Auth email/password |
| `/logout` | Auth | Sign out + clear session |
| `/unauthorized` | Auth | Unauthorized gate page |
| `/dashboard` | App (protected) | Shell home (no business widgets) |
| `/health` | App (protected) | TanStack Query API health |
| `/api/auth/*` | — | Better Auth handler |

## Auth (framework only)

- **Better Auth** with in-memory adapter (no Postgres / Core users yet)
- **JWT** plugin + in-process JWKS; cookie cache strategy `jwt`
- Edge **middleware** gates protected vs guest routes (cookie presence)
- Server layouts re-check session via `auth.api.getSession`

## Stack

- TypeScript + Tailwind CSS + shadcn/ui primitives (`components/ui`)
- TanStack Query, React Hook Form, Zod, Better Auth
- Dark mode via `next-themes`
- Sidebar + top navigation + toasts (Sonner)
- `error.tsx` / `loading.tsx` / `not-found.tsx` / `global-error.tsx`

## Develop

```bash
pnpm --filter @proven/web dev
```
