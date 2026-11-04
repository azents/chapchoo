import { getPublicConfig } from "@/config";
import { AgentsPage } from "@/features/computers/AgentsPage";
import { loadAgents } from "@/features/computers/server/loadAgents";
import type { ReactElement } from "react";

export default async function Page({
  params,
}: {
  params: Promise<{ teamHandle: string; agentHandle: string }>;
}): Promise<ReactElement> {
  const { teamHandle, agentHandle } = await params;
  const initial = await loadAgents(agentHandle, "", teamHandle);
  return (
    <AgentsPage
      basePath={getPublicConfig().publicBasePath}
      key={`${teamHandle}:${agentHandle}`}
      initial={initial}
    />
  );
}
