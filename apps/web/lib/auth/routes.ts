/**
 * Route classification for edge middleware and page guards.
 * AuthZ is not decided here — cookie presence only gates navigation.
 */

export const loginPath = "/login";
export const logoutPath = "/logout";
export const unauthorizedPath = "/unauthorized";

/** Paths that must not require a session cookie. */
const PUBLIC_PREFIXES = [
  "/api/auth",
  "/unauthorized",
  "/_next",
  "/favicon.ico",
  "/icons",
  "/manifest.webmanifest",
] as const;

/** Guest-only paths — authenticated users are redirected away. */
const GUEST_ONLY = [loginPath] as const;

/** Explicit protected app prefixes (foundation shell). */
const PROTECTED_PREFIXES = ["/dashboard", "/health"] as const;

export function isPublicPath(pathname: string): boolean {
  return PUBLIC_PREFIXES.some(
    (prefix) => pathname === prefix || pathname.startsWith(`${prefix}/`),
  );
}

export function isGuestOnlyPath(pathname: string): boolean {
  return GUEST_ONLY.some(
    (path) => pathname === path || pathname.startsWith(`${path}/`),
  );
}

export function isProtectedPath(pathname: string): boolean {
  if (isPublicPath(pathname) || isGuestOnlyPath(pathname)) {
    return false;
  }
  if (pathname === logoutPath) {
    return false;
  }
  return PROTECTED_PREFIXES.some(
    (prefix) => pathname === prefix || pathname.startsWith(`${prefix}/`),
  );
}

export function loginRedirectUrl(origin: string, next?: string): string {
  const url = new URL(loginPath, origin);
  if (next && next.startsWith("/") && !next.startsWith("//")) {
    url.searchParams.set("next", next);
  }
  return url.toString();
}
