import { redirect } from "next/navigation";
import { getLoginRedirect, getPublicConfig } from "@/config/index";
import { AdminGrantsPage } from "@/features/admin-grants/AdminGrantsPage";
import { loadIdentity } from "@/shared/auth/server";
import type { ReactElement } from "react";

export default async function Page(): Promise<ReactElement> {
  const { publicBasePath } = getPublicConfig();
  const initial = await loadIdentity();
  if (initial.type === "LOADED" && initial.data === null) {
    redirect(getLoginRedirect());
  }
  return <AdminGrantsPage kind="reset" basePath={publicBasePath} />;
}
