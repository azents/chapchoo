import "server-only";
import { requiredPageRead } from "@chapchoo/web-core/page-read";
import { call } from "@chapchoo/web-core/rpc";
import { connection } from "next/server";
import { getConfig } from "@/config/index";
import { getBackend } from "@/shared/server/backend";
import type { LoginMethods } from "@chapchoo/web-core/browser";

export async function loadLoginMethods(): Promise<LoginMethods> {
  await connection();
  const backend = getBackend(getConfig());
  return requiredPageRead(() =>
    call(backend.auth.methods.bind(backend.auth), {}),
  );
}

export async function loadSetupStatus(): Promise<{ available: boolean }> {
  await connection();
  const backend = getBackend(getConfig());
  return requiredPageRead(() =>
    call(backend.admin.setupStatus.bind(backend.admin), {}),
  );
}
