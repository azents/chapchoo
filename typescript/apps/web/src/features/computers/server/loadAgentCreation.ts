import "server-only";
import { choices } from "@chapchoo/web-core/computer";
import { RpcFailure } from "@chapchoo/web-core/errors";
import { call } from "@chapchoo/web-core/rpc";
import { team } from "@chapchoo/web-core/validation";
import { notFound, redirect } from "next/navigation";
import { getConfig, getLoginRedirect } from "@/config";
import { readNavigation } from "@/shared/navigation/server";
import { getBackend } from "@/shared/server/backend";
import { readPageScope } from "@/shared/server/identity";
import type { InitialAgentCreation } from "../creationTypes";

export async function loadAgentCreation(
  teamId: string,
): Promise<InitialAgentCreation> {
  const scope = await readPageScope();
  const identity = await scope.identity();
  if (!identity) {
    redirect(getLoginRedirect());
  }
  if (teamId.length > 128) {
    notFound();
  }
  const config = getConfig();
  if (!teamId) {
    const navigation = await readNavigation();
    if (navigation.type !== "ready" || !navigation.preferred) {
      notFound();
    }
    redirect(
      `${config.basePath}/agents/new?team=${encodeURIComponent(navigation.preferred)}`,
    );
  }
  const backend = getBackend(config);
  try {
    await scope.read(async (token) =>
      team(
        await call(
          backend.teams.getTeam.bind(backend.teams),
          { teamId },
          token,
        ),
      ),
    );
    const providers = await scope.read(async (token) =>
      choices(
        await call(
          backend.providerCatalog.list.bind(backend.providerCatalog),
          { limit: 20 },
          token,
        ),
      ),
    );
    return {
      teamId,
      identity: identity.email,
      providers,
      renderedAt: Date.now(),
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
