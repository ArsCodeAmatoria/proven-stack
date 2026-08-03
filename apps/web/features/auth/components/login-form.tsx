"use client";

import { useForm } from "react-hook-form";
import { z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import { useRouter, useSearchParams } from "next/navigation";
import { toast } from "sonner";
import { useState } from "react";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { signIn, signUp } from "@/lib/auth/client";

const loginSchema = z.object({
  email: z.string().email("Enter a valid email"),
  password: z.string().min(8, "Password must be at least 8 characters"),
});

type LoginValues = z.infer<typeof loginSchema>;

function safeNext(raw: string | null): string {
  if (!raw) return "/dashboard";
  if (!raw.startsWith("/") || raw.startsWith("//")) return "/dashboard";
  return raw;
}

/**
 * Better Auth email/password form.
 * Accounts are ephemeral (in-memory) until Core identity lands.
 */
export function LoginForm() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const next = safeNext(searchParams.get("next"));
  const [mode, setMode] = useState<"sign-in" | "scaffold-sign-up">("sign-in");
  const [pending, setPending] = useState(false);

  const form = useForm<LoginValues>({
    resolver: zodResolver(loginSchema),
    defaultValues: { email: "", password: "" },
  });

  async function onSubmit(values: LoginValues) {
    setPending(true);
    try {
      if (mode === "scaffold-sign-up") {
        const result = await signUp.email({
          email: values.email,
          password: values.password,
          name: values.email.split("@")[0] || "Scaffold User",
        });
        if (result.error) {
          toast.error("Could not create scaffold account", {
            description: result.error.message ?? "Try a different email.",
          });
          return;
        }
        toast.success("Scaffold account created", {
          description: "In-memory only — not persisted to Postgres.",
        });
      } else {
        const result = await signIn.email({
          email: values.email,
          password: values.password,
        });
        if (result.error) {
          toast.error("Sign in failed", {
            description: result.error.message ?? "Check email and password.",
          });
          return;
        }
        toast.success("Signed in");
      }
      router.push(next);
      router.refresh();
    } finally {
      setPending(false);
    }
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>{mode === "sign-in" ? "Sign in" : "Scaffold account"}</CardTitle>
        <CardDescription>
          Better Auth framework only — sessions are in-memory. No Core / database
          users yet.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <FormField
              control={form.control}
              name="email"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Email</FormLabel>
                  <FormControl>
                    <Input
                      type="email"
                      autoComplete="username"
                      placeholder="you@company.com"
                      {...field}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="password"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Password</FormLabel>
                  <FormControl>
                    <Input type="password" autoComplete="current-password" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <Button type="submit" className="w-full" disabled={pending}>
              {pending
                ? "Working…"
                : mode === "sign-in"
                  ? "Continue"
                  : "Create scaffold account"}
            </Button>
          </form>
        </Form>
        <div className="flex justify-center">
          <Button
            type="button"
            variant="link"
            className="h-auto p-0 text-sm"
            onClick={() =>
              setMode((m) => (m === "sign-in" ? "scaffold-sign-up" : "sign-in"))
            }
          >
            {mode === "sign-in"
              ? "Need a scaffold account?"
              : "Already have a scaffold account? Sign in"}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
