import { getConfig } from "@/config/index";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

export function GET(): Response {
  // This is web-process/configuration health, not backend or account readiness.
  getConfig();
  return Response.json(
    { status: "ok" },
    {
      headers: { "Cache-Control": "no-store" },
    },
  );
}
