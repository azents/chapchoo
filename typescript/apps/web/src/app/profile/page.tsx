import { redirect } from "next/navigation";
import { getLoginRedirect, getPublicConfig } from "@/config";
import { ProfilePage } from "@/features/preferences/ProfilePage";
import { readIdentity } from "@/shared/server/identity";
import type { ReactElement } from "react";
export default async function Page(): Promise<ReactElement> {
  if (!(await readIdentity())) {
    redirect(getLoginRedirect());
  }
  return <ProfilePage basePath={getPublicConfig().publicBasePath} />;
}
