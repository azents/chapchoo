import { getPublicConfig } from "@/config/index";
import { ComputersPage } from "@/features/computers/ComputersPage";
import { loadComputers } from "@/features/computers/server/loadComputers";
import type { ReactElement } from "react";
export default async function Page({
  params,
  searchParams,
}: {
  params: Promise<{ providerId: string }>;
  searchParams: Promise<{ after?: string | string[] }>;
}): Promise<ReactElement> {
  const { after } = await searchParams;
  const cursor = typeof after === "string" && after.length <= 128 ? after : "";
  return (
    <ComputersPage
      basePath={getPublicConfig().publicBasePath}
      initial={await loadComputers((await params).providerId, cursor)}
    />
  );
}
