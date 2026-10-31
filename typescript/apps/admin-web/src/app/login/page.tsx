import { getPublicConfig } from "@/config/index";
import { AdminAuthPage } from "@/features/admin-auth/AdminAuthPage";
import { loadLoginMethods } from "@/features/admin-auth/server/loadInitialAuth";
import type { ReactElement } from "react";

export default async function Page(): Promise<ReactElement> {
  return (
    <AdminAuthPage
      mode="login"
      basePath={getPublicConfig().publicBasePath}
      initial={await loadLoginMethods()}
    />
  );
}
