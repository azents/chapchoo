import {
  privateResponse,
  reportTransportFailure,
  writableRejection,
} from "@chapchoo/web-core/boundary";
import { fetchRequestHandler } from "@trpc/server/adapters/fetch";
import { getConfig } from "@/config/index";
import { getBackend } from "@/shared/server/backend";
import { createContext } from "@/shared/server/context";
import { appRouter } from "@/shared/server/router";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";
async function handle(request: Request): Promise<Response> {
  const config = getConfig();
  const rejection = writableRejection(request, config);
  if (rejection) {
    return rejection;
  }
  const context = createContext(request, config, getBackend(config));
  const response = await fetchRequestHandler({
    // Next.js removes its configured basePath from the Route Handler URL.
    // The browser link owns the public prefix; this is the internal endpoint.
    endpoint: "/api/trpc",
    req: request,
    router: appRouter,
    createContext: () => context,
    allowMethodOverride: true,
    allowBatching: false,
    onError({ error }) {
      reportTransportFailure(error);
    },
  });
  return privateResponse(response, context.session);
}
export const POST = handle;
export const GET = handle;
