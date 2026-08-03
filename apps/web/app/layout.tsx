import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Proven",
  description: "Construction Compliance Operating System",
  manifest: "/manifest.webmanifest",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
