"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { signOut } from "@/lib/auth/client";
import { loginPath } from "@/lib/auth/routes";

export function LogoutPanel() {
  const router = useRouter();
  const [status, setStatus] = useState<"idle" | "working" | "done" | "error">(
    "idle",
  );

  useEffect(() => {
    let cancelled = false;
    async function run() {
      setStatus("working");
      const result = await signOut();
      if (cancelled) return;
      if (result.error) {
        setStatus("error");
        toast.error("Logout failed", {
          description: result.error.message ?? "Try again.",
        });
        return;
      }
      setStatus("done");
      toast.success("Signed out");
      router.replace(loginPath);
      router.refresh();
    }
    void run();
    return () => {
      cancelled = true;
    };
  }, [router]);

  return (
    <Card>
      <CardHeader>
        <CardTitle>Signing out</CardTitle>
        <CardDescription>
          Ending the Better Auth session and clearing cookies.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        <p className="text-sm text-muted-foreground">
          {status === "working" && "Please wait…"}
          {status === "done" && "Redirecting to sign in…"}
          {status === "error" && "Something went wrong."}
          {status === "idle" && "Starting logout…"}
        </p>
        {status === "error" ? (
          <Button
            onClick={() => {
              setStatus("idle");
              router.refresh();
            }}
          >
            Retry
          </Button>
        ) : null}
      </CardContent>
    </Card>
  );
}
