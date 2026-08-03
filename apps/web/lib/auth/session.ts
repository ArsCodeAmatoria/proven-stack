import { headers } from "next/headers";
import { redirect } from "next/navigation";
import { auth, type Session } from "@/lib/auth/auth";
import { loginPath, unauthorizedPath } from "@/lib/auth/routes";

/** Read the current Better Auth session (RSC / server actions). */
export async function getServerSession(): Promise<Session | null> {
  return auth.api.getSession({
    headers: await headers(),
  });
}

/** Require a session or redirect to login (server components / layouts). */
export async function requireSession(
  nextPath?: string,
): Promise<Session> {
  const session = await getServerSession();
  if (!session) {
    const target = nextPath
      ? `${loginPath}?next=${encodeURIComponent(nextPath)}`
      : loginPath;
    redirect(target);
  }
  return session;
}

/** Redirect authenticated users away from guest-only pages. */
export async function requireGuest(fallback = "/dashboard"): Promise<void> {
  const session = await getServerSession();
  if (session) {
    redirect(fallback);
  }
}

/** Soft guard for pages that should show unauthorized UI instead of login. */
export async function requireAuthorizedSession(): Promise<Session> {
  const session = await getServerSession();
  if (!session) {
    redirect(unauthorizedPath);
  }
  return session;
}
