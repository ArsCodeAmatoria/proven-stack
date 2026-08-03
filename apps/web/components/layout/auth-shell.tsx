import { Fingerprint } from "lucide-react";

export function AuthShell({ children }: { children: React.ReactNode }) {
  return (
    <div className="relative flex min-h-dvh flex-col items-center justify-center px-4 py-10">
      <div className="mb-8 flex items-center gap-3 animate-fade-in">
        <div className="flex h-11 w-11 items-center justify-center rounded-lg bg-primary text-primary-foreground">
          <Fingerprint className="h-5 w-5" aria-hidden />
        </div>
        <div>
          <div className="font-display text-2xl font-semibold tracking-tight">Proven</div>
          <div className="text-sm text-muted-foreground">Construction Compliance OS</div>
        </div>
      </div>
      <div className="w-full max-w-md animate-slide-in">{children}</div>
    </div>
  );
}
