export async function POST(req: Request) {
  const body = await req.json();
  const backendRes = await fetch(
    `${process.env.NEXT_PUBLIC_BACKEND_URL}/query`,
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }
  );
  const data = await backendRes.json();
  return new Response(JSON.stringify(data), { status: backendRes.status });
}