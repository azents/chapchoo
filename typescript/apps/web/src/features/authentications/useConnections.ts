"use client";
import { useAuth } from "@chapchoo/web-core/auth-context";
import { useRouter } from "next/navigation";
import { catalogueOwnerMatches } from "@/shared/authentications/types";
import { useConnectionPanel } from "@/shared/authentications/useConnectionPanel";
import type { ConnectionsInput, ConnectionsProps } from "./types";
export function useConnections(input: ConnectionsInput): ConnectionsProps {
  const auth = useAuth();
  const router = useRouter();
  const path = `${input.basePath}/teams/${encodeURIComponent(input.teamHandle)}/connections`;
  const panel = useConnectionPanel({
    snapshots: [input.snapshot],
    managed: input.managed,
    after: input.after,
    onManage: (record) =>
      router.push(
        `${path}?connection=${encodeURIComponent(record.id)}${input.after ? `&after=${encodeURIComponent(input.after)}` : ""}`,
      ),
    onCloseManage: () =>
      router.push(
        input.after ? `${path}?after=${encodeURIComponent(input.after)}` : path,
      ),
    onPage: (after) =>
      router.push(after ? `${path}?after=${encodeURIComponent(after)}` : path),
  });
  return {
    input,
    panel,
    access: catalogueOwnerMatches(
      input.snapshot,
      auth.state.type === "AUTHENTICATED" ? auth.state.user.id : null,
    ),
    onRetry: () => window.location.reload(),
  };
}
