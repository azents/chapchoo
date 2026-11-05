import { getPublicConfig } from "@/config";
import { ChatPage } from "@/features/computers/chat/ChatPage";
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
    <ChatPage
      basePath={getPublicConfig().publicBasePath}
      key={`${teamHandle}:${agentHandle}`}
      initial={initial}
    />
  );
}
