import Link from "next/link";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { loginPath } from "@/lib/auth/routes";

export function UnauthorizedPanel() {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Unauthorized</CardTitle>
        <CardDescription>
          You need an active session to view that page. AuthZ decisions will live
          in Rust Core later — this is the AuthN gate only.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-wrap gap-3">
        <Button asChild>
          <Link href={loginPath}>Sign in</Link>
        </Button>
        <Button asChild variant="outline">
          <Link href="/">Home</Link>
        </Button>
      </CardContent>
    </Card>
  );
}
