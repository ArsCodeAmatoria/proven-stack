import type { Metadata } from "next";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

export const metadata: Metadata = {
  title: "Dashboard",
};

export default function DashboardPage() {
  return (
    <div className="space-y-6 animate-fade-in">
      <div>
        <p className="text-xs font-medium uppercase tracking-[0.14em] text-muted-foreground">
          Foundation
        </p>
        <h2 className="mt-1 font-display text-3xl font-semibold tracking-tight">Dashboard</h2>
        <p className="mt-2 max-w-2xl text-sm text-muted-foreground">
          App shell is ready: sidebar, top navigation, dark mode, toasts, and layouts.
          Business features are intentionally not implemented.
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Shell</CardTitle>
            <CardDescription>Authenticated layout primitives</CardDescription>
          </CardHeader>
          <CardContent className="text-sm text-muted-foreground">
            Sidebar + top nav + responsive sheet menu.
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Platform health</CardTitle>
            <CardDescription>API connectivity probe</CardDescription>
          </CardHeader>
          <CardContent>
            <Button asChild variant="secondary">
              <Link href="/health">Open health page</Link>
            </Button>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
