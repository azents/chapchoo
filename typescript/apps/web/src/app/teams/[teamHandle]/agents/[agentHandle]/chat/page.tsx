import { getPublicConfig } from "@/config";
import { ChatPage } from "@/features/computers/chat/ChatPage";
import { loadChat } from "@/features/computers/chat/server/loadChat";
import type { ReactElement } from "react";

export default async function Page({
  params,
}: {
  params: Promise<{ teamHandle: string; agentHandle: string }>;
}): Promise<ReactElement> {
  const { teamHandle, agentHandle } = await params;
  const initial = await loadChat(agentHandle, teamHandle);
  return (
    <ChatPage
      basePath={getPublicConfig().publicBasePath}
      key={`${teamHandle}:${agentHandle}`}
      {...initial}
    />
  );
}
