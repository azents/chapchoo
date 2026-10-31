import { getPublicConfig } from "@/config/index";
import { AdminGrantsPage } from "@/features/admin-grants/AdminGrantsPage";
import { loadInventory } from "@/features/admin-grants/server/loadInventory";
import type { ReactElement } from "react";

export default async function Page({
  searchParams,
}: {
  searchParams: Promise<{ offset?: string | string[] }>;
}): Promise<ReactElement> {
  const { publicBasePath } = getPublicConfig();
  return (
    <AdminGrantsPage
      kind="reset"
      basePath={publicBasePath}
      initial={await loadInventory("reset", (await searchParams).offset)}
    />
  );
}
