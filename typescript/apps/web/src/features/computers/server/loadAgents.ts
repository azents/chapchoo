import "server-only";
import { agent, agents } from "@chapchoo/web-core/computer";
import { RpcFailure } from "@chapchoo/web-core/errors";
import { call } from "@chapchoo/web-core/rpc";
import { notFound, redirect } from "next/navigation";
import { getConfig, getLoginRedirect } from "@/config";
import { resolveRouteTeam } from "@/shared/navigation/team";
import { agentAuthenticationScopes } from "@/shared/server/authenticationSnapshots";
import { getBackend } from "@/shared/server/backend";
import { readPageScope } from "@/shared/server/identity";
import type { InitialAgents } from "../types";

export async function loadAgents(
  agentHandle: string,
  after: string,
  teamHandle: string,
): Promise<InitialAgents> {
  const { team: selected, identity } = await resolveRouteTeam(teamHandle);
  const teamId = selected.id;
  const scope = await readPageScope();
  const config = getConfig();
  const backend = getBackend(config);
  try {
    const common = {
      renderedAt: Date.now(),
      identity: identity.email,
      teamHandle,
    };
    if (agentHandle) {
      const current = await scope.read(async (token) =>
        agent(
          await call(
            backend.agents.getInTeam.bind(backend.agents),
            { teamId, handle: agentHandle },
            token,
          ),
        ),
      );
      if (current.teamId !== teamId) {
        notFound();
      }
      return {
        ...common,
        teamId,
        type: "detail",
        agent: current,
        authenticationSnapshots: agentAuthenticationScopes(
          selected,
          identity.id,
        ),
      };
    }
    return {
      ...common,
      teamId,
      type: "list",
      after,
      page: await scope.read(async (token) =>
        agents(
          await call(
            backend.agents.list.bind(backend.agents),
            { teamId, page: { after: after || void 0, limit: 20 } },
            token,
          ),
        ),
      ),
    };
  } catch (error) {
    if (
      error instanceof RpcFailure &&
      (error.reason === "DOMAIN_REASON_NOT_FOUND" ||
        error.reason === "DOMAIN_REASON_INVALID_INPUT")
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
