import "server-only";
import { teamKind } from "@chapchoo/web-core/browser";
import { RpcFailure } from "@chapchoo/web-core/errors";
import { validHandle } from "@chapchoo/web-core/handle";
import { call } from "@chapchoo/web-core/rpc";
import { team, teams } from "@chapchoo/web-core/validation";
import { notFound, redirect } from "next/navigation";
import { cache } from "react";
import { getConfig, getLoginRedirect } from "@/config";
import { getBackend } from "@/shared/server/backend";
import { readPageScope } from "@/shared/server/identity";
import type { Team, User } from "@chapchoo/web-core/browser";

/** Authentication precedes route syntax and accessible-Team resolution. */
export const resolveRouteTeam = cache(
  async (handle: string): Promise<{ team: Team; identity: User }> => {
    const scope = await readPageScope();
    const identity = await scope.identity();
    if (!identity) {
      redirect(getLoginRedirect());
    }
    if (handle !== "my" && !validHandle(handle)) {
      notFound();
    }
    const backend = getBackend(getConfig());
    try {
      const selected = await scope.read(async (token) => {
        if (handle === "my") {
          const contexts = teams(
            await call(backend.teams.listTeams.bind(backend.teams), {}, token),
          );
          return (
            contexts.items.find((item) => item.kind === teamKind.personal) ??
            null
          );
        }
        return team(
          await call(
            backend.teams.resolveHandle.bind(backend.teams),
            { handle },
            token,
          ),
        );
      });
      if (!selected) {
        notFound();
      }
      return { team: selected, identity };
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
  },
);
