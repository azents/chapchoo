import { logoutResponse } from "@chapchoo/web-core/logout";
import { call } from "@chapchoo/web-core/rpc";
import { after } from "next/server";
import { getConfig } from "@/config/index";
import { getBackend } from "@/shared/server/backend";

export function POST(request: Request): Response {
  const config = getConfig();
  return logoutResponse(
    request,
    config,
    async (token) => {
      const backend = getBackend(config);
      await call(backend.auth.logout.bind(backend.auth), {}, token);
    },
    after,
  );
}
