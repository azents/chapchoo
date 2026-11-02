import "server-only";
import { resources } from "@chapchoo/web-core/computer";
import { call } from "@chapchoo/web-core/rpc";
import { redirect } from "next/navigation";
import { getConfig, getLoginRedirect } from "@/config/index";
import { getBackend } from "@/shared/server/backend";
import { readPageScope } from "@/shared/server/identity";
import type { InitialResources } from "../resourceTypes";
import type { ResourceSelection } from "@/shared/server/resourceInput";
export async function loadResources(
  selection: ResourceSelection,
): Promise<InitialResources> {
  const scope = await readPageScope();
  const identity = await scope.identity();
  if (!identity) {
    redirect(getLoginRedirect());
  }
  const backend = getBackend(getConfig());
  return {
    selection,
    identity: identity.email,
    renderedAt: Date.now(),
    page: await scope.read(async (token) =>
      resources(
        await call(
          backend.providerInventory.listResources.bind(
            backend.providerInventory,
          ),
          { ...selection, after: selection.after, limit: 20 },
          token,
        ),
      ),
    ),
  };
}
