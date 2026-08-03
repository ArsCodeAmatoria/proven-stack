import { Button } from "@proven/ui";
import { createApiClient } from "@proven/api-client";

const apiBase =
  process.env.NEXT_PUBLIC_PROVEN_API_URL ?? "http://127.0.0.1:8080";

export default async function HomePage() {
  let apiStatus = "unreachable";
  try {
    const client = createApiClient({ baseUrl: apiBase });
    const health = await client.health();
    apiStatus = `${health.data.status} (v${health.data.version})`;
  } catch {
    apiStatus = "unreachable (start proven-api on :8080)";
  }

  return (
    <main
      style={{
        maxWidth: 720,
        margin: "0 auto",
        padding: "4rem 1.5rem",
      }}
    >
      <p style={{ letterSpacing: "0.12em", textTransform: "uppercase", color: "var(--muted)", fontSize: 12 }}>
        Foundation
      </p>
      <h1 style={{ fontSize: "3rem", margin: "0.5rem 0 0.75rem", fontWeight: 650 }}>
        Proven
      </h1>
      <p style={{ color: "var(--muted)", lineHeight: 1.6, maxWidth: 42 + "rem" }}>
        Construction Compliance Operating System — monorepo foundation is live.
        Business modules come next.
      </p>
      <div
        style={{
          marginTop: "2rem",
          padding: "1.25rem 1.5rem",
          background: "var(--surface)",
          borderRadius: 12,
          border: "1px solid #2a3544",
        }}
      >
        <div style={{ fontSize: 13, color: "var(--muted)" }}>API health</div>
        <div style={{ marginTop: 6, fontFamily: "ui-monospace, monospace" }}>
          {apiStatus}
        </div>
      </div>
      <div style={{ marginTop: "1.5rem" }}>
        <Button>Open My Actions (soon)</Button>
      </div>
    </main>
  );
}
