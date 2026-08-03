/**
 * Better Auth server instance (AuthN framework only).
 *
 * - No Postgres / Core user tables yet — uses an in-memory adapter.
 * - JWT plugin is wired with an in-process JWKS store (replace with Core/KMS later).
 * - Permissions must never appear in JWT claims (AuthZ stays in Rust Core).
 */

import { betterAuth } from "better-auth";
import { memoryAdapter, type MemoryDB } from "better-auth/adapters/memory";
import { nextCookies } from "better-auth/next-js";
import { jwt } from "better-auth/plugins";
import type { Jwk } from "better-auth/plugins/jwt";

/** Process-local store — foundation only; lost on restart. */
const memoryDb: MemoryDB = {};

/** In-memory JWKS for the JWT plugin (no DB jwks table yet). */
const jwksStore: Jwk[] = [];

function requireAuthSecret(): string {
  const secret =
    process.env.BETTER_AUTH_SECRET?.trim() ||
    process.env.PROVEN_SESSION_SECRET?.trim();
  if (secret && secret.length >= 32) {
    return secret;
  }
  // `next build` sets NODE_ENV=production; gate on Proven env instead.
  if (process.env.PROVEN_ENV === "production") {
    throw new Error(
      "BETTER_AUTH_SECRET (or PROVEN_SESSION_SECRET) must be set (≥ 32 chars) in production",
    );
  }
  return "dev-only-better-auth-secret-change-me-32b";
}

function authBaseURL(): string {
  return (
    process.env.BETTER_AUTH_URL?.trim() ||
    process.env.NEXT_PUBLIC_APP_URL?.trim() ||
    "http://localhost:3000"
  );
}

export const auth = betterAuth({
  appName: "Proven",
  secret: requireAuthSecret(),
  baseURL: authBaseURL(),
  database: memoryAdapter(memoryDb),
  emailAndPassword: {
    enabled: true,
    // Foundation scaffold: allow ephemeral local accounts in memory only.
    disableSignUp: false,
  },
  session: {
    expiresIn: 60 * 60 * 24 * 7,
    updateAge: 60 * 60 * 24,
    cookieCache: {
      enabled: true,
      maxAge: 5 * 60,
      strategy: "jwt",
    },
  },
  plugins: [
    jwt({
      jwt: {
        issuer: authBaseURL(),
        audience: "proven-api",
        expirationTime: "15m",
        definePayload: ({ user, session }) => ({
          // Access claims only — no roles / permissions (Core AuthZ later).
          sub: user.id,
          sid: session.id,
          email_verified: Boolean(user.emailVerified),
        }),
      },
      adapter: {
        getJwks: async () => jwksStore,
        createJwk: async (webKey) => {
          const row: Jwk = {
            ...webKey,
            id: crypto.randomUUID(),
          };
          jwksStore.push(row);
          return row;
        },
      },
    }),
    nextCookies(),
  ],
});

export type Session = typeof auth.$Infer.Session;
