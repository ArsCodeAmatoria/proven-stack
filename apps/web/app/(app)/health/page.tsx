import type { Metadata } from "next";
import { HealthPanel } from "@/components/health/health-panel";

export const metadata: Metadata = {
  title: "Health",
};

export default function HealthPage() {
  return (
    <div className="mx-auto max-w-xl space-y-6 animate-fade-in">
      <div>
        <p className="text-xs font-medium uppercase tracking-[0.14em] text-muted-foreground">
          Platform
        </p>
        <h2 className="mt-1 font-display text-3xl font-semibold tracking-tight">Health</h2>
        <p className="mt-2 text-sm text-muted-foreground">
          Live check of the Rust API. Start <code className="font-mono text-xs">proven-api</code> on
          :8080 for a successful response.
        </p>
      </div>
      <HealthPanel />
    </div>
  );
}
