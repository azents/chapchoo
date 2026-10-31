import { getPublicConfig } from "@/config";
import { AuthPage } from "@/features/accounts/AuthPage";
import { loadInitialAuth } from "@/features/accounts/server/loadInitialAuth";
import type { ReactElement } from "react";

export default async function Login(): Promise<ReactElement> {
  return (
    <AuthPage
      basePath={getPublicConfig().publicBasePath}
      mode="LOGIN"
      token=""
      initial={await loadInitialAuth("LOGIN", "")}
    />
  );
}
