import "server-only";
import { agent, agents } from "@chapchoo/web-core/computer";
import { RpcFailure } from "@chapchoo/web-core/errors";
import { call } from "@chapchoo/web-core/rpc";
import { notFound, redirect } from "next/navigation";
import { getConfig, getLoginRedirect } from "@/config";
import { getBackend } from "@/shared/server/backend";
import { readPageScope } from "@/shared/server/identity";
import type { InitialAgents } from "../types";

export async function loadAgents(
  agentId: string,
  after: string,
): Promise<InitialAgents> {
  const scope = await readPageScope();
  const identity = await scope.identity();
  if (!identity) {
    redirect(getLoginRedirect());
  }
  const backend = getBackend(getConfig());
  try {
    const common = { renderedAt: Date.now(), identity: identity.email };
    if (agentId) {
      return {
        ...common,
        type: "detail",
        agent: await scope.read(async (token) =>
          agent(
            await call(
              backend.agents.get.bind(backend.agents),
              { agentId },
              token,
            ),
          ),
        ),
      };
    }
    return {
      ...common,
      type: "list",
      after,
      page: await scope.read(async (token) =>
        agents(
          await call(
            backend.agents.list.bind(backend.agents),
            { after: after || void 0, limit: 20 },
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
