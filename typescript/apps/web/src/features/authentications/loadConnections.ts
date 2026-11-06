import "server-only";
import { teamKind, teamRole } from "@chapchoo/web-core/browser";
import {
  authentication,
  wireAuthenticationScope,
} from "@chapchoo/web-core/external-authentication";
import { call } from "@chapchoo/web-core/rpc";
import { notFound } from "next/navigation";
import { getConfig, getPublicConfig } from "@/config";
import { resolveRouteTeam } from "@/shared/navigation/team";
import { readAuthenticationSnapshot } from "@/shared/server/authenticationSnapshots";
import { getBackend } from "@/shared/server/backend";
import { readPageScope } from "@/shared/server/identity";
import type { ConnectionsInput } from "./types";
import type { AuthenticationScope } from "@chapchoo/web-core/external-authentication";
import type { SearchValues } from "@chapchoo/web-core/page-state";
export async function loadConnections(
  teamHandle: string,
  params: SearchValues,
): Promise<ConnectionsInput> {
  const { team, identity } = await resolveRouteTeam(teamHandle);
  const scope: AuthenticationScope =
    team.kind === teamKind.personal
      ? { type: "personal" }
      : { type: "team", teamId: team.id };
  const canManage =
    team.kind === teamKind.personal ||
    team.callerRole === teamRole.owner ||
    team.callerRole === teamRole.manager;
  const after = params.get("after");
  const connection = params.get("connection");
  if (connection && !canManage) {
    notFound();
  }
  if (
    (after !== null && (!after || after.length > 128)) ||
    (connection !== null && (!connection || connection.length > 128))
  ) {
    notFound();
  }
  const snapshot = await readAuthenticationSnapshot(
    scope,
    canManage,
    identity.id,
    after ?? void 0,
  );
  const reader = await readPageScope();
  const backend = getBackend(getConfig());
  const managed = connection
    ? await reader.read(async (token) =>
        authentication(
          await call(
            backend.authentications.get.bind(backend.authentications),
            {
              scope: wireAuthenticationScope(scope),
              authenticationId: connection,
            },
            token,
          ),
        ),
      )
    : null;
  return {
    basePath: getPublicConfig().publicBasePath,
    teamHandle,
    identity: identity.email,
    snapshot,
    managed,
    after: after ?? null,
  };
}
