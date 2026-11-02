import { getPublicConfig } from "@/config";
import { AgentsPage } from "@/features/computers/AgentsPage";
import { loadAgents } from "@/features/computers/server/loadAgents";
import type { ReactElement } from "react";
export default async function Page({
  params,
  searchParams,
}: {
  params: Promise<{ agentId: string }>;
  searchParams: Promise<{ team?: string | string[] }>;
}): Promise<ReactElement> {
  const { team } = await searchParams;
  return (
    <AgentsPage
      basePath={getPublicConfig().publicBasePath}
      initial={await loadAgents(
        (await params).agentId,
        "",
        typeof team === "string" ? team : "",
      )}
    />
  );
}
