import { teamKind } from "./browser.js";
import type { Team } from "./browser.js";
import { validHandle } from "./handle.js";
import type { ShellCopy } from "./shell-messages.js";

export function profileRoute(route: string): boolean {
  return route === "/profile" || route.startsWith("/profile/");
}
export interface ProfileNavigationItem {
  key: string;
  label: string;
  href: string;
  active: boolean;
  icon: "profile" | "info" | "mail";
}
export function profileNavigation(
  mount: string,
  route: string,
  copy: ShellCopy,
  invitations: boolean,
): ProfileNavigationItem[] {
  return [
    {
      key: "account",
      label: copy.accountSettings,
      href: `${mount}/profile`,
      active: route === "/profile",
      icon: "profile",
    },
    {
      key: "display",
      label: copy.displaySettings,
      href: `${mount}/profile/preferences`,
      active: route === "/profile/preferences",
      icon: "info",
    },
    ...(invitations
      ? [
          {
            key: "invitations",
            label: copy.invitations,
            href: `${mount}/profile/invitations`,
            active: route === "/profile/invitations",
            icon: "mail" as const,
          },
        ]
      : []),
  ];
}

/** Public route selector. IDs remain preference/cache keys, never path content. */
export function teamRouteKey(team: Team): string {
  if (team.kind === teamKind.personal) return "my";
  if (!team.handle || !validHandle(team.handle))
    throw new Error("Collaborative Team handle is invalid");
  return team.handle;
}
/** Mirror the framework's decoded path parameter without accepting ID queries. */
export function routeTeamHandle(path: string): string | null {
  const segment = /^\/teams\/([^/]+)(?:\/|$)/.exec(path)?.[1];
  if (!segment) return null;
  try {
    const handle = decodeURIComponent(segment);
    return handle === "my" || validHandle(handle) ? handle : null;
  } catch {
    return null;
  }
}
export function teamPath(mount: string, team: Team | string): string {
  const handle = typeof team === "string" ? team : teamRouteKey(team);
  if (handle !== "my" && !validHandle(handle))
    throw new Error("Team route handle is invalid");
  return `${mount.replace(/\/$/, "")}/teams/${encodeURIComponent(handle)}`;
}
export function agentsPath(mount: string, team: Team | string): string {
  return `${teamPath(mount, team)}/agents`;
}
export function newAgentPath(mount: string, team: Team | string): string {
  return `${teamPath(mount, team)}/new-agent`;
}
export function agentPath(
  mount: string,
  team: Team | string,
  handle: string,
): string {
  if (!validHandle(handle)) throw new Error("Agent route handle is invalid");
  return `${agentsPath(mount, team)}/${encodeURIComponent(handle)}`;
}

export function agentChatPath(
  mount: string,
  team: Team | string,
  handle: string,
): string {
  return `${agentPath(mount, team, handle)}/chat`;
}

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
