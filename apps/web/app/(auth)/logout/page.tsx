import type { Metadata } from "next";
import { LogoutPanel } from "@/features/auth/components/logout-panel";

export const metadata: Metadata = {
  title: "Sign out",
};

export default function LogoutPage() {
  return <LogoutPanel />;
}
