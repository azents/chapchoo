import "server-only";
import { cookiePolicy, readSessionCookie } from "@chapchoo/web-core/cookies";
import { call } from "@chapchoo/web-core/rpc";
import { RequestSession } from "@chapchoo/web-core/session";
import type { WebConfig } from "@chapchoo/web-core/config";
import type { BackendClients } from "@chapchoo/web-core/rpc";

export interface RpcContext {
  session: RequestSession;
  backend: BackendClients;
}
export function createContext(
  request: Request,
  config: WebConfig,
  backend: BackendClients,
): RpcContext {
  const policy = cookiePolicy(config);
  const session = new RequestSession(
    {
      renew: (token) => call(backend.auth.renew.bind(backend.auth), { token }),
      identity: (token) =>
        call(backend.auth.identity.bind(backend.auth), {}, token),
    },
    readSessionCookie(request.headers.get("cookie"), policy),
    policy,
    config.app === "admin",
  );
  return { session, backend };
}
