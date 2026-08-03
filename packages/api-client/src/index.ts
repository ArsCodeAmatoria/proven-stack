export type HealthResponse = {
  data: {
    status: string;
    version: string;
  };
};

export type ApiClientOptions = {
  baseUrl: string;
  fetch?: typeof fetch;
};

/** Thin HTTP client for `/api/v1`. No business rules. */
export function createApiClient(options: ApiClientOptions) {
  const fetchFn = options.fetch ?? fetch;
  const base = options.baseUrl.replace(/\/$/, "");

  return {
    async health(): Promise<HealthResponse> {
      const res = await fetchFn(`${base}/api/v1/health`);
      if (!res.ok) {
        throw new Error(`health check failed: ${res.status}`);
      }
      return res.json() as Promise<HealthResponse>;
    },
  };
}
