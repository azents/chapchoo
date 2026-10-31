import "server-only";
import { cookiePolicy, readSessionCookie } from "@chapchoo/web-core/cookies";
import { call } from "@chapchoo/web-core/rpc";
import { readOnlyIdentity } from "@chapchoo/web-core/session";
import { headers } from "next/headers";
import { cache } from "react";
import { getConfig } from "@/config/index";
import { getBackend } from "@/shared/server/backend";
import type { User } from "@chapchoo/web-core/browser";

export const readIdentity = cache(async (): Promise<User | null> => {
  const config = getConfig();
  const backend = getBackend(config);
  const policy = cookiePolicy(config);
  const token = readSessionCookie((await headers()).get("cookie"), policy);
  return readOnlyIdentity(
    {
      identity: (credential) =>
        call(backend.auth.identity.bind(backend.auth), {}, credential),
    },
    token,
    config.app === "admin",
  );
});
