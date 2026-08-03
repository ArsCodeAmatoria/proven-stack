import { describe, expect, it } from "vitest";
import { loginPath, logoutPath, unauthorizedPath } from "@/lib/auth/routes";

describe("auth routes", () => {
  it("exposes foundation auth paths", () => {
    expect(loginPath).toBe("/login");
    expect(logoutPath).toBe("/logout");
    expect(unauthorizedPath).toBe("/unauthorized");
  });
});
