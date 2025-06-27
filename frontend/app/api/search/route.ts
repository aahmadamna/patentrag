// app/api/search/route.ts

import { NextRequest, NextResponse } from "next/server";

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const resp = await fetch("http://localhost:3000/search", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
    const data = await resp.json();
    return NextResponse.json(data, { status: resp.status });
  } catch (err: unknown) {
    if (err instanceof Error) {
      console.error("❌ Proxy /api/search error:", err.message);
    } else {
      console.error("❌ Proxy /api/search error:", err);
    }
    return NextResponse.json({ error: "Proxy failed" }, { status: 500 });
  }
}
