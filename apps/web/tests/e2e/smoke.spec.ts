import { test, expect } from "@playwright/test";

test.describe("foundation smoke", () => {
  test("login page renders", async ({ page }) => {
    await page.goto("/login");
    await expect(page.getByRole("heading", { name: /sign in|scaffold/i })).toBeVisible({
      timeout: 30_000,
    });
  });

  test("api health route responds", async ({ request }) => {
    const res = await request.get("/api/health");
    expect(res.ok()).toBeTruthy();
    const body = await res.json();
    expect(body.service).toBe("proven-web");
  });
});
