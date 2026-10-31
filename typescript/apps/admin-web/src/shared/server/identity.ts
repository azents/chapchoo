import "server-only";
import { cookiePolicy, readSessionCookie } from "@chapchoo/web-core/cookies";
import { PageReadScope } from "@chapchoo/web-core/page-read";
import { call } from "@chapchoo/web-core/rpc";
import { headers } from "next/headers";
import { cache } from "react";
import { getConfig } from "@/config/index";
import { getBackend } from "@/shared/server/backend";
import type { User } from "@chapchoo/web-core/browser";

export const readPageScope = cache(async (): Promise<PageReadScope> => {
  const config = getConfig();
  const backend = getBackend(config);
  const policy = cookiePolicy(config);
  const token = readSessionCookie((await headers()).get("cookie"), policy);
  return new PageReadScope(
    token,
    (credential) =>
      call(backend.auth.identity.bind(backend.auth), {}, credential),
    config.app === "admin",
  );
});
export const readIdentity = cache(async (): Promise<User | null> =>
  (await readPageScope()).identity(),
);
