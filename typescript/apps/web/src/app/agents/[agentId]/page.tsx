import { getPublicConfig } from "@/config";
import { AgentsPage } from "@/features/computers/AgentsPage";
import { loadAgents } from "@/features/computers/server/loadAgents";
import type { ReactElement } from "react";
export default async function Page({
  params,
}: {
  params: Promise<{ agentId: string }>;
}): Promise<ReactElement> {
  return (
    <AgentsPage
      basePath={getPublicConfig().publicBasePath}
      initial={await loadAgents((await params).agentId, "")}
    />
  );
}
