import { getPublicConfig } from "@/config";
import { AuthPage } from "@/features/accounts/AuthPage";
import { loadInitialAuth } from "@/features/accounts/server/loadInitialAuth";
import type { ReactElement } from "react";

export default async function ResetPassword({
  searchParams,
}: {
  searchParams: Promise<{ token?: string }>;
}): Promise<ReactElement> {
  const { token = "" } = await searchParams;
  return (
    <AuthPage
      basePath={getPublicConfig().publicBasePath}
      mode="RESET"
      token={token}
      initial={await loadInitialAuth("RESET", token)}
    />
  );
}
