import "server-only";
import { inventory, providers } from "@chapchoo/web-core/computer";
import { call } from "@chapchoo/web-core/rpc";
import { redirect } from "next/navigation";
import { getConfig, getLoginRedirect } from "@/config/index";
import { getBackend } from "@/shared/server/backend";
import { readPageScope } from "@/shared/server/identity";
import type { InitialComputers } from "../types";
export async function loadComputers(
  providerId: string,
  after: string,
): Promise<InitialComputers> {
  const scope = await readPageScope();
  const identity = await scope.identity();
  if (!identity) {
    redirect(getLoginRedirect());
  }
  const backend = getBackend(getConfig());
  const common = { after, renderedAt: Date.now(), identity: identity.email };
  if (!providerId) {
    return {
      ...common,
      type: "providers",
      page: await scope.read(async (token) =>
        providers(
          await call(
            backend.providerInventory.listProviders.bind(
              backend.providerInventory,
            ),
            { after: after || void 0, limit: 20 },
            token,
          ),
        ),
      ),
    };
  }
  return {
    ...common,
    type: "computers",
    providerId,
    page: await scope.read(async (token) =>
      inventory(
        await call(
          backend.providerInventory.listComputers.bind(
            backend.providerInventory,
          ),
          { providerId, page: { after: after || void 0, limit: 20 } },
          token,
        ),
      ),
    ),
  };
}
