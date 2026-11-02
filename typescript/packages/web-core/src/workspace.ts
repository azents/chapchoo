import { teamKind } from "./browser.js";
import type { Team } from "./browser.js";

export const workspaceCookie = "chapchoo-workspace";
export function preferredTeam(
  value: string | undefined,
  userId: string,
  teams: readonly Team[],
): Team | null {
  if (value && value.length <= 1024) {
    try {
      const parsed: unknown = JSON.parse(decodeURIComponent(value));
      if (
        typeof parsed === "object" &&
        parsed !== null &&
        "user" in parsed &&
        parsed.user === userId &&
        "team" in parsed &&
        typeof parsed.team === "string"
      ) {
        const selected = teams.find((team) => team.id === parsed.team);
        if (selected) return selected;
      }
    } catch {
      /* Malformed browser preference has no admission authority. */
    }
  }
  return teams.find((team) => team.kind === teamKind.personal) ?? null;
}
export function workspacePreference(
  userId: string,
  teamId: string,
  secure: boolean,
): string {
  return `${workspaceCookie}=${encodeURIComponent(JSON.stringify({ user: userId, team: teamId }))}; Path=/; SameSite=Lax; Max-Age=2592000${secure ? "; Secure" : ""}`;
}
