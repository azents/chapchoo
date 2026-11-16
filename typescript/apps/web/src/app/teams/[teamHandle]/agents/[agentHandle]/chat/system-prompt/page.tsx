import { getPublicConfig } from "@/config";
import { InstructionEvidencePage } from "@/features/computers/chat/InstructionEvidencePage";
import { loadSystemPrompt } from "@/features/computers/chat/server/loadSystemPrompt";
import type { ReactElement } from "react";

export default async function Page({
  params,
}: {
  params: Promise<{ teamHandle: string; agentHandle: string }>;
}): Promise<ReactElement> {
  const { teamHandle, agentHandle } = await params;
  const initial = await loadSystemPrompt(agentHandle, teamHandle);
  return (
    <InstructionEvidencePage
      basePath={getPublicConfig().publicBasePath}
      {...initial}
    />
  );
}
