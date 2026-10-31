import "server-only";
import { loadInitialRead } from "@chapchoo/web-core/initial-read";
import { call } from "@chapchoo/web-core/rpc";
import { connection } from "next/server";
import { getConfig } from "@/config/index";
import { getBackend } from "@/shared/server/backend";
import type { InitialRead, LoginMethods } from "@chapchoo/web-core/browser";

export async function loadLoginMethods(): Promise<InitialRead<LoginMethods>> {
  await connection();
  const backend = getBackend(getConfig());
  return loadInitialRead(() =>
    call(backend.auth.methods.bind(backend.auth), {}),
  );
}

export async function loadSetupStatus(): Promise<
  InitialRead<{ available: boolean }>
> {
  await connection();
  const backend = getBackend(getConfig());
  return loadInitialRead(() =>
    call(backend.admin.setupStatus.bind(backend.admin), {}),
  );
}
