import { redirect } from "next/navigation";
import { getLoginRedirect, getPublicConfig } from "@/config";
import { WorkspacePage } from "@/features/accounts/WorkspacePage";
import { readIdentity } from "@/shared/server/identity";
import type { ReactElement } from "react";

export default async function Team({
  params,
}: {
  params: Promise<{ teamId: string }>;
}): Promise<ReactElement> {
  const { teamId } = await params;
  if (!(await readIdentity())) {
    redirect(getLoginRedirect());
  }
  return (
    <WorkspacePage
      basePath={getPublicConfig().publicBasePath}
      key={teamId}
      teamId={teamId}
    />
  );
}
