import { redirect } from "next/navigation";
import { getLoginRedirect, getPublicConfig } from "@/config";
import { WorkspacePage } from "@/features/accounts/WorkspacePage";
import { readIdentity } from "@/shared/server/identity";
import type { ReactElement } from "react";

export default async function Home(): Promise<ReactElement> {
  if (!(await readIdentity())) {
    redirect(getLoginRedirect());
  }
  return (
    <WorkspacePage basePath={getPublicConfig().publicBasePath} teamId="" />
  );
}
