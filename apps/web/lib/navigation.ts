import {
  Activity,
  LayoutDashboard,
  type LucideIcon,
} from "lucide-react";

export type NavItem = {
  title: string;
  href: string;
  icon: LucideIcon;
};

/** Foundation navigation only — no domain modules. */
export const appNav: NavItem[] = [
  { title: "Dashboard", href: "/dashboard", icon: LayoutDashboard },
  { title: "Health", href: "/health", icon: Activity },
];
