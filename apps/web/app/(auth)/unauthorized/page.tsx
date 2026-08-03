import type { Metadata } from "next";
import { UnauthorizedPanel } from "@/features/auth/components/unauthorized-panel";

export const metadata: Metadata = {
  title: "Unauthorized",
};

export default function UnauthorizedPage() {
  return <UnauthorizedPanel />;
}
