import type { McpServerView } from "@chapchoo/web-core/computer";

export interface McpDraft {
  displayName: string;
  endpoint: string;
  auth: "none" | "bearer";
  bearer: string;
}
export type McpConnection = { endpoint: string } & {
  auth: { type: "none" | "keep" } | { type: "replace"; bearer: string };
};

export function draftFor(server: McpServerView | null): McpDraft {
  return {
    displayName: server?.displayName ?? "",
    endpoint: server?.endpoint ?? "",
    auth: server?.hasBearer ? "bearer" : "none",
    bearer: "",
  };
}

export function mcpConnection(
  draft: McpDraft,
  saved: McpServerView | null,
): McpConnection | null {
  const url = URL.parse(draft.endpoint.trim());
  if (
    !url ||
    url.protocol !== "https:" ||
    !url.hostname ||
    url.username ||
    url.password ||
    url.href.includes("#") ||
    url.href.length > 4096
  ) {
    return null;
  }
  const endpoint = url.href;
  if (draft.auth === "none") {
    return { endpoint, auth: { type: "none" } };
  }
  if (/^[!-~]{1,8192}$/u.test(draft.bearer)) {
    return { endpoint, auth: { type: "replace", bearer: draft.bearer } };
  }
  if (!draft.bearer && saved?.hasBearer && endpoint === saved.endpoint) {
    return { endpoint, auth: { type: "keep" } };
  }
  return null;
}

export function knownRejection(reason?: string | null): boolean {
  return (
    reason === "DOMAIN_REASON_INVALID_INPUT" ||
    reason === "DOMAIN_REASON_INVALID_TRANSITION" ||
    reason === "DOMAIN_REASON_INVALID_SESSION" ||
    reason === "DOMAIN_REASON_PERMISSION_DENIED" ||
    reason === "DOMAIN_REASON_NOT_FOUND"
  );
}
