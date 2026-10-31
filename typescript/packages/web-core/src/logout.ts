import {
  clearSessionCookie,
  cookiePolicy,
  readSessionCookie,
} from "./cookies.js";
import { reportRpcFailure, rpcFailure } from "./errors.js";
import type { WebConfig } from "./config.js";

/** Local credential disposal is independent of best-effort backend revocation. */
export function logoutResponse(
  request: Request,
  config: WebConfig,
  revoke: (token: string) => Promise<void>,
  after: (task: () => Promise<void>) => void,
): Response {
  if (request.method !== "POST")
    return new Response(null, { status: 405, headers: { allow: "POST" } });
  if (request.headers.get("origin") !== config.origin) {
    return Response.json(
      { error: "A same-origin request is required" },
      { status: 403 },
    );
  }
  const policy = cookiePolicy(config);
  const token = readSessionCookie(request.headers.get("cookie"), policy);
  if (token !== null) {
    after(async () => {
      try {
        await revoke(token);
      } catch (error) {
        reportRpcFailure(rpcFailure(error));
      }
    });
  }
  return new Response(null, {
    status: 303,
    headers: {
      "set-cookie": clearSessionCookie(policy),
      location: `${config.origin}${config.basePath}/login`,
      "cache-control": "no-store",
    },
  });
}
