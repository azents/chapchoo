import { identityManifest } from "@chapchoo/ui/webIdentity";
import { getPublicConfig } from "@/config/index";

export const dynamic = "force-dynamic";

export function GET(): Response {
  return Response.json(
    identityManifest(getPublicConfig().publicBasePath, "Chapchoo Admin"),
    { headers: { "Content-Type": "application/manifest+json" } },
  );
}
