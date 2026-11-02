import { getPublicConfig } from "@/config";
import { AgentsPage } from "@/features/computers/AgentsPage";
import { loadAgents } from "@/features/computers/server/loadAgents";
import type { ReactElement } from "react";
export default async function Page({
  searchParams,
}: {
  searchParams: Promise<{
    after?: string | string[];
    team?: string | string[];
  }>;
}): Promise<ReactElement> {
  const { after, team } = await searchParams;
  const cursor = typeof after === "string" && after.length <= 128 ? after : "";
  return (
    <AgentsPage
      basePath={getPublicConfig().publicBasePath}
      initial={await loadAgents(
        "",
        cursor,
        typeof team === "string" ? team : "",
      )}
    />
  );
}
