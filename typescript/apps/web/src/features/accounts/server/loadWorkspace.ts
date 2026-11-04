import "server-only";
import { teamKind } from "@chapchoo/web-core/browser";
import { RpcFailure } from "@chapchoo/web-core/errors";
import { call } from "@chapchoo/web-core/rpc";
import { invitations, members } from "@chapchoo/web-core/validation";
import { notFound, redirect } from "next/navigation";
import { getConfig, getLoginRedirect } from "@/config";
import { resolveRouteTeam } from "@/shared/navigation/team";
import { getBackend } from "@/shared/server/backend";
import { readPageScope } from "@/shared/server/identity";
import type { InitialWorkspace } from "../initialWorkspace";

export async function loadWorkspace(
  teamHandle: string,
): Promise<InitialWorkspace> {
  const scope = await readPageScope();
  if (!(await scope.identity())) {
    redirect(getLoginRedirect());
  }
  const backend = getBackend(getConfig());
  try {
    if (!teamHandle) {
      return {
        type: "INVITATIONS",
        received: await scope.read(async (token) =>
          invitations(
            await call(
              backend.invitations.receivedInvitations.bind(backend.invitations),
              {},
              token,
            ),
          ),
        ),
      };
    }
    const { team: selected } = await resolveRouteTeam(teamHandle);
    const teamId = selected.id;
    const membership =
      selected.kind === teamKind.collaborative
        ? await scope.read(async (token) =>
            members(
              await call(
                backend.teams.listMembers.bind(backend.teams),
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
