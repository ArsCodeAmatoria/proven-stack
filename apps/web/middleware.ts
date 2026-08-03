import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";
import { getSessionCookie } from "better-auth/cookies";
import {
  isGuestOnlyPath,
  isProtectedPath,
  loginPath,
  loginRedirectUrl,
} from "@/lib/auth/routes";

/**
 * Edge auth gate — cookie presence only (optimistic redirect).
 * Always re-validate sessions on the server for protected actions.
 */
export function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;
  const sessionCookie = getSessionCookie(request);
  const hasSession = Boolean(sessionCookie);

  if (isProtectedPath(pathname) && !hasSession) {
    return NextResponse.redirect(
      loginRedirectUrl(request.nextUrl.origin, pathname),
    );
  }

  if (isGuestOnlyPath(pathname) && hasSession) {
    return NextResponse.redirect(new URL("/dashboard", request.url));
  }

  if (pathname === "/" && !hasSession) {
    return NextResponse.redirect(new URL(loginPath, request.url));
  }

  return NextResponse.next();
}

export const config = {
  matcher: ["/((?!_next/static|_next/image|favicon.ico|icons|manifest.webmanifest).*)"],
};
