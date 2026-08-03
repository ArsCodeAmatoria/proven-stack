"use client";

import { useEffect } from "react";
import { Button } from "@/components/ui/button";

export default function AppError({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  useEffect(() => {
    console.error(error);
  }, [error]);

  return (
    <div className="space-y-3">
      <h2 className="font-display text-xl font-semibold">Section error</h2>
      <p className="text-sm text-muted-foreground">{error.message}</p>
      <Button type="button" onClick={reset}>
        Retry
      </Button>
    </div>
  );
}
