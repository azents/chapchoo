import { redirect } from "next/navigation";
import { getLoginRedirect, getPublicConfig } from "@/config";
import { PreferencesPage } from "@/features/preferences/PreferencesPage";
import { readIdentity } from "@/shared/server/identity";
import type { ReactElement } from "react";
export default async function Page(): Promise<ReactElement> {
  if (!(await readIdentity())) {
    redirect(getLoginRedirect());
  }
  return <PreferencesPage basePath={getPublicConfig().publicBasePath} />;
}
