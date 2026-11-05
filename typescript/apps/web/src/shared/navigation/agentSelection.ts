import { validHandle } from "@chapchoo/web-core/handle";
import { selectionUrl } from "@chapchoo/web-core/page-state";
import type { AgentView } from "@chapchoo/web-core/computer";
import type { UrlCodec } from "@chapchoo/web-core/page-state";

export const agentCursorKey = "agentAfter";
export const agentCursor: UrlCodec<string | null> = {
  parse(value) {
    return value && value.length <= 128 ? value : null;
  },
  format(value) {
    return this.parse(value);
  },
};

export function agentNavigationHref(
  href: string,
  after: string | null,
): string {
  return selectionUrl(href, agentCursorKey, after, agentCursor);
}

export function routeAgentHandle(route: string): string | null {
  const segment = /^\/teams\/[^/]+\/agents\/([^/]+)(?:\/|$)/.exec(route)?.[1];
  if (!segment) {
    return null;
  }
  try {
    const handle = decodeURIComponent(segment);
    return validHandle(handle) ? handle : null;
  } catch {
    return null;
  }
}

/** Both sources use existing private-owner admission; never mix Team snapshots. */
export function agentNavigationItems(
  page: readonly AgentView[],
  current: AgentView | null,
  teamId: string,
  handle: string | null,
): readonly AgentView[] {
  const items = page.filter((agent) => agent.teamId === teamId);
  return current &&
    current.teamId === teamId &&
    current.handle === handle &&
    !items.some((agent) => agent.id === current.id)
    ? [current, ...items]
    : items;
}
