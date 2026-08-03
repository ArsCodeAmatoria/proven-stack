/**
 * Typed web / Node configuration for Proven.
 * Validates environment, required public URLs, and missing keys.
 * No business domain logic.
 */

export type Environment = "development" | "testing" | "production";

export type WebConfig = {
  environment: Environment;
  /** Browser-facing API origin */
  publicApiUrl: string;
  /** Server-side API origin (RSC / route handlers) */
  serverApiUrl: string;
};

export class ConfigError extends Error {
  readonly code: "missing" | "invalid" | "secrets" | "startup";
  readonly details: string[];

  constructor(
    code: ConfigError["code"],
    message: string,
    details: string[] = [],
  ) {
    super(details.length ? `${message}: ${details.join("; ")}` : message);
    this.name = "ConfigError";
    this.code = code;
    this.details = details;
  }
}

export type EnvSource = Record<string, string | undefined>;

export function parseEnvironment(raw: string | undefined): Environment {
  const value = (raw ?? "development").trim().toLowerCase();
  switch (value) {
    case "development":
    case "dev":
    case "local":
      return "development";
    case "testing":
    case "test":
      return "testing";
    case "production":
    case "prod":
      return "production";
    default:
      throw new ConfigError(
        "invalid",
        "invalid PROVEN_ENV",
        [`unknown environment '${raw}' (expected development|testing|production)`],
      );
  }
}

/**
 * Load and validate web configuration from an env-like object.
 * Defaults: `process.env` when `source` is omitted (Node / Next server).
 */
export function loadWebConfig(source?: EnvSource): WebConfig {
  const env = source ?? (typeof process !== "undefined" ? process.env : {});
  const environment = parseEnvironment(env.PROVEN_ENV);

  const missing: string[] = [];
  const publicApiUrl = pickUrl(
    env.NEXT_PUBLIC_PROVEN_API_URL,
    environment === "development" ? "http://127.0.0.1:8080" : undefined,
    "NEXT_PUBLIC_PROVEN_API_URL",
    missing,
  );
  const serverApiUrl = pickUrl(
    env.PROVEN_API_URL ?? env.NEXT_PUBLIC_PROVEN_API_URL,
    environment === "development" ? "http://127.0.0.1:8080" : undefined,
    "PROVEN_API_URL",
    missing,
  );

  if (missing.length) {
    throw new ConfigError(
      "missing",
      "missing required configuration",
      missing,
    );
  }

  const config: WebConfig = {
    environment,
    publicApiUrl: publicApiUrl!,
    serverApiUrl: serverApiUrl!,
  };

  validateWebSecrets(config);
  validateWebStartup(config);
  return config;
}

export function validateWebSecrets(config: WebConfig): void {
  const reasons: string[] = [];
  if (config.environment === "production") {
    if (isLoopback(config.publicApiUrl)) {
      reasons.push(
        "NEXT_PUBLIC_PROVEN_API_URL must not be localhost in production",
      );
    }
  }
  if (reasons.length) {
    throw new ConfigError("secrets", "secrets validation failed", reasons);
  }
}

export function validateWebStartup(config: WebConfig): void {
  const reasons: string[] = [];
  for (const [key, value] of [
    ["NEXT_PUBLIC_PROVEN_API_URL", config.publicApiUrl],
    ["PROVEN_API_URL", config.serverApiUrl],
  ] as const) {
    try {
      // eslint-disable-next-line no-new
      new URL(value);
    } catch {
      reasons.push(`${key} is not a valid URL`);
    }
  }
  if (reasons.length) {
    throw new ConfigError("startup", "startup validation failed", reasons);
  }
}

function pickUrl(
  value: string | undefined,
  fallback: string | undefined,
  key: string,
  missing: string[],
): string | undefined {
  if (value && value.trim()) return value.trim().replace(/\/$/, "");
  if (fallback) return fallback.replace(/\/$/, "");
  missing.push(key);
  return undefined;
}

function isLoopback(url: string): boolean {
  try {
    const host = new URL(url).hostname;
    return (
      host === "localhost" ||
      host === "127.0.0.1" ||
      host === "::1" ||
      host === "0.0.0.0"
    );
  } catch {
    return false;
  }
}
