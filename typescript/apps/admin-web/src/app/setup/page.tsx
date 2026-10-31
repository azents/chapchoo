import { getPublicConfig } from "@/config/index";
import { AdminAuthPage } from "@/features/admin-auth/AdminAuthPage";
import { loadSetupStatus } from "@/features/admin-auth/server/loadInitialAuth";
import type { ReactElement } from "react";

export default async function Page(): Promise<ReactElement> {
  return (
    <AdminAuthPage
      mode="setup"
      basePath={getPublicConfig().publicBasePath}
      initial={await loadSetupStatus()}
    />
  );
}
