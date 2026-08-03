import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  ConfigError,
  loadWebConfig,
  parseEnvironment,
} from "./index.js";

describe("parseEnvironment", () => {
  it("accepts aliases", () => {
    assert.equal(parseEnvironment("dev"), "development");
    assert.equal(parseEnvironment("test"), "testing");
    assert.equal(parseEnvironment("prod"), "production");
  });

  it("rejects unknown", () => {
    assert.throws(() => parseEnvironment("staging"), ConfigError);
  });
});

describe("loadWebConfig", () => {
  it("uses development defaults", () => {
    const cfg = loadWebConfig({ PROVEN_ENV: "development" });
    assert.equal(cfg.environment, "development");
    assert.equal(cfg.publicApiUrl, "http://127.0.0.1:8080");
  });

  it("detects missing production keys", () => {
    assert.throws(
      () => loadWebConfig({ PROVEN_ENV: "production" }),
      (err: unknown) =>
        err instanceof ConfigError &&
        err.code === "missing" &&
        err.details.includes("NEXT_PUBLIC_PROVEN_API_URL"),
    );
  });

  it("rejects localhost public URL in production", () => {
    assert.throws(
      () =>
        loadWebConfig({
          PROVEN_ENV: "production",
          NEXT_PUBLIC_PROVEN_API_URL: "http://localhost:8080",
          PROVEN_API_URL: "http://api.internal:8080",
        }),
      (err: unknown) => err instanceof ConfigError && err.code === "secrets",
    );
  });

  it("accepts production urls", () => {
    const cfg = loadWebConfig({
      PROVEN_ENV: "production",
      NEXT_PUBLIC_PROVEN_API_URL: "https://api.proven.example",
      PROVEN_API_URL: "http://api:8080",
    });
    assert.equal(cfg.environment, "production");
  });
});
