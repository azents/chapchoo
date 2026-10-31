import { randomInt } from "node:crypto";
import { mascotIds } from "@chapchoo/ui/designTokens";
import { getPublicConfig } from "@/config/index";
import { AdminAuthPage } from "@/features/admin-auth/AdminAuthPage";
import { loadLoginMethods } from "@/features/admin-auth/server/loadInitialAuth";
import type { ReactElement } from "react";

export default async function Page(): Promise<ReactElement> {
  const initial = await loadLoginMethods();
  return (
    <AdminAuthPage
      mode="login"
      character={mascotIds[randomInt(mascotIds.length)]}
      basePath={getPublicConfig().publicBasePath}
      initial={initial}
    />
  );
}
