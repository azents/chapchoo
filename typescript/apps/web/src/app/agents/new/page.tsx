import { getPublicConfig } from "@/config";
import { AgentCreationPage } from "@/features/computers/AgentCreationPage";
import { loadAgentCreation } from "@/features/computers/server/loadAgentCreation";
import type { ReactElement } from "react";

export default async function Page({
  searchParams,
}: {
  searchParams: Promise<{ team?: string | string[] }>;
}): Promise<ReactElement> {
  const { team } = await searchParams;
  const initial = await loadAgentCreation(typeof team === "string" ? team : "");
  return (
    <AgentCreationPage
      basePath={getPublicConfig().publicBasePath}
      key={initial.teamId}
      initial={initial}
    />
  );
}
