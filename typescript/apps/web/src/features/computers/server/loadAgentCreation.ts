import "server-only";
import { randomInt } from "node:crypto";
import { choices, profileImages } from "@chapchoo/web-core/computer";
import { RpcFailure } from "@chapchoo/web-core/errors";
import { call } from "@chapchoo/web-core/rpc";
import { notFound, redirect } from "next/navigation";
import { getConfig, getLoginRedirect } from "@/config";
import { resolveRouteTeam } from "@/shared/navigation/team";
import { getBackend } from "@/shared/server/backend";
import { readPageScope } from "@/shared/server/identity";
import { initialProfile } from "../creationDefaults";
import type { InitialAgentCreation } from "../creationTypes";

export async function loadAgentCreation(
  teamHandle: string,
): Promise<InitialAgentCreation> {
  const { team: selected, identity } = await resolveRouteTeam(teamHandle);
  const teamId = selected.id;
  const scope = await readPageScope();
  const config = getConfig();
  const backend = getBackend(config);
  try {
    const providers = await scope.read(async (token) =>
      choices(
        await call(
          backend.providerCatalog.list.bind(backend.providerCatalog),
          { limit: 20 },
          token,
        ),
      ),
    );
    const images = await scope.read(async (token) =>
      profileImages(
        await call(
          backend.agents.listProfileImages.bind(backend.agents),
          { limit: 20 },
          token,
        ),
      ),
    );
    // Presentation-only draft shared by SSR and hydration, never a create default.
    const selection = initialProfile(images.items, (bound) => randomInt(bound));
    return {
      teamId,
      teamHandle,
      identity: identity.email,
      providers,
      images,
      ...selection,
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
