"use client";

import { createAuthClient } from "better-auth/react";
import { jwtClient } from "better-auth/client/plugins";

/**
 * Browser Better Auth client.
 * Base URL defaults to same origin (`/api/auth`).
 */
export const authClient = createAuthClient({
  // jwtClient typings lag createAuthClient plugin arity in 1.6.x — cast keeps scaffold typed.
  plugins: [jwtClient() as never],
});

export const { signIn, signOut, signUp, useSession, getSession } = authClient;
