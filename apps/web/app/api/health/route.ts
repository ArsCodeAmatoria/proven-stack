import { NextResponse } from "next/server";

/** Process liveness for the Next.js app (no dependency checks). */
export async function GET() {
  return NextResponse.json(
    {
      status: "ok",
      service: "proven-web",
    },
    {
      status: 200,
      headers: {
        "Cache-Control": "no-store",
      },
    },
  );
}
