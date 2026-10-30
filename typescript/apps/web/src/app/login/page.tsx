import { getPublicConfig } from "@/config";
import { AuthPage } from "@/features/accounts/AuthPage";
import type { ReactElement } from "react";

export default function Login(): ReactElement {
  return (
    <AuthPage
      basePath={getPublicConfig().publicBasePath}
      mode="LOGIN"
      token=""
    />
  );
}
