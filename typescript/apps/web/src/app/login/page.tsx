import { randomInt } from "node:crypto";
import { mascotIds } from "@chapchoo/ui/designTokens";
import { getPublicConfig } from "@/config";
import { AuthPage } from "@/features/accounts/AuthPage";
import { loadInitialAuth } from "@/features/accounts/server/loadInitialAuth";
import type { ReactElement } from "react";

export default async function Login(): Promise<ReactElement> {
  const initial = await loadInitialAuth("LOGIN", "");
  return (
    <AuthPage
      basePath={getPublicConfig().publicBasePath}
      mode="LOGIN"
      character={mascotIds[randomInt(mascotIds.length)]}
      token=""
      initial={initial}
    />
  );
}
