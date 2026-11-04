import { getPublicConfig } from "@/config";
import { loadWorkspace } from "@/features/accounts/server/loadWorkspace";
import { WorkspacePage } from "@/features/accounts/WorkspacePage";
import type { ReactElement } from "react";

export default async function Team({
  params,
}: {
  params: Promise<{ teamHandle: string }>;
}): Promise<ReactElement> {
  const { teamHandle } = await params;
  const initial = await loadWorkspace(teamHandle);
  return (
    <WorkspacePage
      basePath={getPublicConfig().publicBasePath}
      key={teamHandle}
      teamId={initial.type === "TEAM" ? initial.team.id : ""}
      initial={initial}
    />
  );
}
