import { getPublicConfig } from "@/config/index";
import { AdminGrantsPage } from "@/features/admin-grants/AdminGrantsPage";
import { loadAdminIdentity } from "@/features/admin-grants/server/loadAdminIdentity";
import type { ReactElement } from "react";

export default async function Page(): Promise<ReactElement> {
  const { publicBasePath } = getPublicConfig();
  const initial = await loadAdminIdentity();
  return (
    <AdminGrantsPage
      kind="signup"
      basePath={publicBasePath}
      initial={initial}
    />
  );
}
