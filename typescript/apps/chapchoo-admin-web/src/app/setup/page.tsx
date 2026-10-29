import { getPublicConfig } from "@/config/index";
import { AdminAuthPage } from "@/features/admin-auth/AdminAuthPage";
import type { ReactElement } from "react";

export default function Page(): ReactElement {
  return (
    <AdminAuthPage mode="setup" basePath={getPublicConfig().publicBasePath} />
  );
}
