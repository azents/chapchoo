import { getPublicConfig } from "@/config";
import { loadWorkspace } from "@/features/accounts/server/loadWorkspace";
import { WorkspacePage } from "@/features/accounts/WorkspacePage";
import type { ReactElement } from "react";

export default async function Invitations(): Promise<ReactElement> {
  return (
    <WorkspacePage
      basePath={getPublicConfig().publicBasePath}
      teamId=""
      initial={await loadWorkspace("")}
    />
  );
}
