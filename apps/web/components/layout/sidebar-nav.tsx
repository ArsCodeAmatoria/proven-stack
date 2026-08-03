"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { Fingerprint } from "lucide-react";
import { cn } from "@/lib/utils";
import { appNav } from "@/lib/navigation";
import { Separator } from "@/components/ui/separator";

export function SidebarNav({ onNavigate }: { onNavigate?: () => void }) {
  const pathname = usePathname();

  return (
    <div className="flex h-full flex-col">
      <div className="flex h-14 items-center gap-2 px-4">
        <div className="flex h-8 w-8 items-center justify-center rounded-md bg-primary text-primary-foreground">
          <Fingerprint className="h-4 w-4" aria-hidden />
        </div>
        <div className="min-w-0">
          <div className="font-display text-sm font-semibold tracking-tight">Proven</div>
          <div className="truncate text-xs text-muted-foreground">Foundation shell</div>
        </div>
      </div>
      <Separator />
      <nav className="flex-1 space-y-1 p-3" aria-label="Primary">
        {appNav.map((item) => {
          const active =
            pathname === item.href || pathname.startsWith(`${item.href}/`);
          const Icon = item.icon;
          return (
            <Link
              key={item.href}
              href={item.href}
              onClick={onNavigate}
              className={cn(
                "flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors",
                active
                  ? "bg-sidebar-accent text-foreground"
                  : "text-muted-foreground hover:bg-sidebar-accent hover:text-foreground",
              )}
            >
              <Icon className="h-4 w-4" aria-hidden />
              {item.title}
            </Link>
          );
        })}
      </nav>
      <div className="p-4 text-xs text-muted-foreground">
        No business modules yet.
      </div>
    </div>
  );
}
