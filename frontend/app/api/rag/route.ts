// app/api/rag/route.ts

import { NextRequest, NextResponse } from "next/server";

export async function POST(request: NextRequest) {
  try {
    // Grab whatever JSON body the client sent
    const body = await request.json();

    // Forward to your Rust /query endpoint
    const resp = await fetch("http://localhost:3000/query", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });

    // Mirror status + JSON back to the client
    const data = await resp.json();
    return NextResponse.json(data, { status: resp.status });
  } catch (err: unknown) {
    if (err instanceof Error) {
      console.error("❌ Proxy /api/rag error:", err.message);
    } else {
      console.error("❌ Proxy /api/rag error:", err);
    }
    return NextResponse.json(
      { error: "Proxy failed" },
      { status: 500 }
    );
  }
}
