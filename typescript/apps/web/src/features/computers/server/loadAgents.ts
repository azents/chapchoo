import "server-only";
import { agent, agents } from "@chapchoo/web-core/computer";
import { RpcFailure } from "@chapchoo/web-core/errors";
import { call } from "@chapchoo/web-core/rpc";
import { notFound, redirect } from "next/navigation";
import { getConfig, getLoginRedirect } from "@/config";
import { readNavigation } from "@/shared/navigation/server";
import { getBackend } from "@/shared/server/backend";
import { readPageScope } from "@/shared/server/identity";
import type { InitialAgents } from "../types";

export async function loadAgents(
  agentId: string,
  after: string,
  teamId: string,
): Promise<InitialAgents> {
  const scope = await readPageScope();
  const identity = await scope.identity();
  if (!identity) {
    redirect(getLoginRedirect());
  }
  if (teamId.length > 128) {
    notFound();
  }
  const config = getConfig();
  const backend = getBackend(config);
  try {
    const common = { renderedAt: Date.now(), identity: identity.email };
    if (agentId) {
      const current = await scope.read(async (token) =>
        agent(
          await call(
            backend.agents.get.bind(backend.agents),
            { agentId },
            token,
          ),
        ),
      );
      if (!teamId) {
        redirect(
          `${config.basePath}/agents/${encodeURIComponent(agentId)}?team=${encodeURIComponent(current.teamId)}`,
        );
      }
      if (current.teamId !== teamId) {
        notFound();
      }
      return { ...common, teamId, type: "detail", agent: current };
    }
    if (!teamId) {
      const context = await readNavigation();
      if (context.type !== "ready" || !context.preferred) {
        notFound();
      }
      redirect(
        `${config.basePath}/agents?team=${encodeURIComponent(context.preferred)}${after ? `&after=${encodeURIComponent(after)}` : ""}`,
      );
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
