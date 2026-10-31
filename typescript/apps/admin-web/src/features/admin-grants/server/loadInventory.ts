import "server-only";
import {
  inventoryOffset,
  readSelection,
  serverSearchValues,
} from "@chapchoo/web-core/page-state";
import { call } from "@chapchoo/web-core/rpc";
import { grants } from "@chapchoo/web-core/validation";
import { redirect } from "next/navigation";
import { getConfig, getLoginRedirect } from "@/config/index";
import { getBackend } from "@/shared/server/backend";
import { readPageScope } from "@/shared/server/identity";
import type { GrantPageKind } from "../components/AdminGrants";
import type { InitialInventory } from "../initialInventory";

export async function loadInventory(
  kind: GrantPageKind,
  value?: string | string[],
): Promise<InitialInventory> {
  const scope = await readPageScope();
  if (!(await scope.identity())) {
    redirect(getLoginRedirect());
  }
  const offset = readSelection(
    serverSearchValues({ offset: value }),
    "offset",
    inventoryOffset,
  );
  const backend = getBackend(getConfig());
  const page = await scope.read(async (token) =>
    grants(
      await call(
        kind === "signup"
          ? backend.admin.listSignupGrants.bind(backend.admin)
          : backend.admin.listPasswordResets.bind(backend.admin),
        { offset, limit: 25n },
        token,
      ),
    ),
  );
  return { offset: offset.toString(), page, renderedAt: Date.now() };
}
