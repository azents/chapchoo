import "server-only";
import { call } from "@chapchoo/web-core/rpc";
import { teams } from "@chapchoo/web-core/validation";
import { preferredTeam, workspaceCookie } from "@chapchoo/web-core/workspace";
import { cookies } from "next/headers";
import { cache } from "react";
import { getConfig } from "@/config";
import { getBackend } from "@/shared/server/backend";
import { readPageScope } from "@/shared/server/identity";
import type { InitialNavigation } from "./types";

export const readNavigation = cache(async (): Promise<InitialNavigation> => {
  const scope = await readPageScope();
  const identity = await scope.identity();
  if (!identity) {
    return { type: "guest" };
  }
  try {
    const backend = getBackend(getConfig());
    const contexts = await scope.read(async (token) =>
      teams(await call(backend.teams.listTeams.bind(backend.teams), {}, token)),
    );
    const preference = (await cookies()).get(workspaceCookie)?.value;
    return {
      type: "ready",
      userId: identity.id,
      teams: contexts.items,
      preferred:
        preferredTeam(preference, identity.id, contexts.items)?.id ?? null,
      renderedAt: Date.now(),
    };
  } catch {
    // Navigation is supplementary; an unavailable selector never replaces
    // independently server-rendered primary content with an empty success.
    return { type: "error", userId: identity.id };
  }
});
