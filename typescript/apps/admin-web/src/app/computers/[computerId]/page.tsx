import { getPublicConfig } from "@/config/index";
import { ResourcesPage } from "@/features/computers/ResourcesPage";
import { loadResources } from "@/features/computers/server/loadResources";
import { resourceSelection } from "@/shared/server/resourceInput";
import type { ReactElement } from "react";
export default async function Page({
  params,
  searchParams,
}: {
  params: Promise<{ computerId: string }>;
  searchParams: Promise<{
    kind?: string | string[];
    namespace?: string | string[];
    name?: string | string[];
  }>;
}): Promise<ReactElement> {
  const selection = resourceSelection(
    (await params).computerId,
    await searchParams,
  );
  return (
    <ResourcesPage
      basePath={getPublicConfig().publicBasePath}
      initial={await loadResources(selection)}
    />
  );
}
