import { getPublicConfig } from "@/config";
import { AuthPage } from "@/features/accounts/AuthPage";
import type { ReactElement } from "react";

export default async function Signup({
  searchParams,
}: {
  searchParams: Promise<{ token?: string }>;
}): Promise<ReactElement> {
  const { token = "" } = await searchParams;
  return (
    <AuthPage
      basePath={getPublicConfig().publicBasePath}
      mode="SIGNUP"
      token={token}
    />
  );
}
