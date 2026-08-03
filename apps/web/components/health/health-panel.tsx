"use client";

import { useQuery } from "@tanstack/react-query";
import { createApiClient } from "@proven/api-client";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { toast } from "sonner";

function apiBaseUrl() {
  return (
    process.env.NEXT_PUBLIC_PROVEN_API_URL?.replace(/\/$/, "") ??
    "http://127.0.0.1:8080"
  );
}

async function fetchHealth() {
  const client = createApiClient({ baseUrl: apiBaseUrl() });
  return client.health();
}

export function HealthPanel() {
  const query = useQuery({
    queryKey: ["platform", "health"],
    queryFn: fetchHealth,
  });

  return (
    <Card>
      <CardHeader>
        <CardTitle>API health</CardTitle>
        <CardDescription>
          TanStack Query probe against <code className="font-mono text-xs">/api/v1/health</code>.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {query.isLoading ? (
          <div className="space-y-2">
            <Skeleton className="h-4 w-40" />
            <Skeleton className="h-4 w-64" />
          </div>
        ) : query.isError ? (
          <p className="text-sm text-destructive">
            {query.error instanceof Error ? query.error.message : "Health check failed"}
          </p>
        ) : (
          <dl className="grid gap-2 text-sm">
            <div className="flex justify-between gap-4">
              <dt className="text-muted-foreground">Status</dt>
              <dd className="font-mono">{query.data?.data.status}</dd>
            </div>
            <div className="flex justify-between gap-4">
              <dt className="text-muted-foreground">Version</dt>
              <dd className="font-mono">{query.data?.data.version}</dd>
            </div>
          </dl>
        )}
        <Button
          type="button"
          variant="secondary"
          onClick={() => {
            void query.refetch();
            toast.message("Refreshing health");
          }}
        >
          Refresh
        </Button>
      </CardContent>
    </Card>
  );
}
