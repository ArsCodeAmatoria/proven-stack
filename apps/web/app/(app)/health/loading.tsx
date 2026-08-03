import { Skeleton } from "@/components/ui/skeleton";

export default function HealthLoading() {
  return (
    <div className="mx-auto max-w-xl space-y-4">
      <Skeleton className="h-8 w-40" />
      <Skeleton className="h-4 w-72 max-w-full" />
      <Skeleton className="h-48 w-full" />
    </div>
  );
}
