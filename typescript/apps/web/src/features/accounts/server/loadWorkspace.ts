import "server-only";
import { teamKind } from "@chapchoo/web-core/browser";
import { RpcFailure } from "@chapchoo/web-core/errors";
import { call } from "@chapchoo/web-core/rpc";
import { members, team, teams } from "@chapchoo/web-core/validation";
import { notFound, redirect } from "next/navigation";
import { getConfig, getLoginRedirect } from "@/config";
import { getBackend } from "@/shared/server/backend";
import { readPageScope } from "@/shared/server/identity";
import type { InitialWorkspace } from "../initialWorkspace";

export async function loadWorkspace(teamId: string): Promise<InitialWorkspace> {
  const scope = await readPageScope();
  if (!(await scope.identity())) {
    redirect(getLoginRedirect());
  }
  const backend = getBackend(getConfig());
  try {
    if (!teamId) {
      return {
        type: "HOME",
        contexts: await scope.read(async (token) =>
          teams(
            await call(
              backend.general.listTeams.bind(backend.general),
              {},
              token,
            ),
          ),
        ),
      };
    }
    const selected = await scope.read(async (token) =>
      team(
        await call(
          backend.general.getTeam.bind(backend.general),
          { teamId },
          token,
        ),
      ),
    );
    const membership =
      selected.kind === teamKind.collaborative
        ? await scope.read(async (token) =>
            members(
              await call(
                backend.general.listMembers.bind(backend.general),
                { teamId },
                token,
              ),
            ),
          )
        : { items: [] };
    return { type: "TEAM", team: selected, members: membership };
  } catch (error) {
    if (
      error instanceof RpcFailure &&
      error.reason === "DOMAIN_REASON_NOT_FOUND"
    ) {
      notFound();
    }
    if (
      error instanceof RpcFailure &&
      error.reason === "DOMAIN_REASON_INVALID_SESSION"
    ) {
      redirect(getLoginRedirect());
    }
    throw error;
  }
}
