import { getPublicConfig } from "@/config";
import { AgentCreationPage } from "@/features/computers/AgentCreationPage";
import { loadAgentCreation } from "@/features/computers/server/loadAgentCreation";
import type { ReactElement } from "react";

export default async function Page({
  params,
}: {
  params: Promise<{ teamHandle: string }>;
}): Promise<ReactElement> {
  const { teamHandle } = await params;
  return (
    <AgentCreationPage
      basePath={getPublicConfig().publicBasePath}
      key={teamHandle}
      initial={await loadAgentCreation(teamHandle)}
    />
  );
}
