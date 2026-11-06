import "server-only";
import { teamKind, teamRole } from "@chapchoo/web-core/browser";
import {
  authenticationList,
  wireAuthenticationScope,
} from "@chapchoo/web-core/external-authentication";
import { call } from "@chapchoo/web-core/rpc";
import { getConfig } from "@/config";
import { getBackend } from "@/shared/server/backend";
import { readPageScope } from "@/shared/server/identity";
import type { CatalogueSnapshot } from "@/shared/authentications/types";
import type { Team } from "@chapchoo/web-core/browser";
import type { AuthenticationScope } from "@chapchoo/web-core/external-authentication";

export async function readAuthenticationSnapshot(
  scope: AuthenticationScope,
  canManage: boolean,
  ownerUserId: string,
  after?: string,
): Promise<CatalogueSnapshot> {
  const reader = await readPageScope();
  const backend = getBackend(getConfig());
  const page = await reader.read(async (token) =>
    authenticationList(
      await call(
        backend.authentications.list.bind(backend.authentications),
        { scope: wireAuthenticationScope(scope), after, limit: 20 },
        token,
      ),
    ),
  );
  return { scope, canManage, ownerUserId, page, renderedAt: Date.now() };
}
export function agentAuthenticationScopes(
  team: Team,
  ownerUserId: string,
): CatalogueSnapshot[] {
  const personal: CatalogueSnapshot = {
    scope: { type: "personal" },
    canManage: true,
    ownerUserId,
    page: null,
    renderedAt: Date.now(),
  };
  if (team.kind === teamKind.personal) {
    return [personal];
  }
  return [
    personal,
    {
      scope: { type: "team", teamId: team.id },
      canManage:
        team.callerRole === teamRole.owner ||
        team.callerRole === teamRole.manager,
      ownerUserId,
      page: null,
      renderedAt: Date.now(),
    },
  ];
}
export async function readAgentAuthenticationSnapshots(
  team: Team,
  ownerUserId: string,
): Promise<CatalogueSnapshot[]> {
  return Promise.all(
    agentAuthenticationScopes(team, ownerUserId).map((value) =>
      readAuthenticationSnapshot(value.scope, value.canManage, ownerUserId),
    ),
  );
}
