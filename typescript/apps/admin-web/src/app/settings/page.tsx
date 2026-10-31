import { redirect } from "next/navigation";
import { getLoginRedirect, getPublicConfig } from "@/config";
import { ServerSettingsPage } from "@/features/preferences/ServerSettingsPage";
import { readDisplay } from "@/shared/server/display";
import { readIdentity } from "@/shared/server/identity";
import type { ReactElement } from "react";
export default async function Page(): Promise<ReactElement> {
  if (!(await readIdentity())) {
    redirect(getLoginRedirect());
  }
  return (
    <ServerSettingsPage
      basePath={getPublicConfig().publicBasePath}
      initial={(await readDisplay()).defaults}
    />
  );
}
