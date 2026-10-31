import { getPublicConfig } from "@/config";
import { loadWorkspace } from "@/features/accounts/server/loadWorkspace";
import { WorkspacePage } from "@/features/accounts/WorkspacePage";
import type { ReactElement } from "react";

export default async function Team({
  params,
}: {
  params: Promise<{ teamId: string }>;
}): Promise<ReactElement> {
  const { teamId } = await params;
  return (
    <WorkspacePage
      basePath={getPublicConfig().publicBasePath}
      key={teamId}
      teamId={teamId}
      initial={await loadWorkspace(teamId)}
    />
  );
}
