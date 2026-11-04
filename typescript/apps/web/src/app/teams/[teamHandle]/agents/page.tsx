import { getPublicConfig } from "@/config";
import { AgentsPage } from "@/features/computers/AgentsPage";
import { loadAgents } from "@/features/computers/server/loadAgents";
import type { ReactElement } from "react";

export default async function Page({
  params,
  searchParams,
}: {
  params: Promise<{ teamHandle: string }>;
  searchParams: Promise<{ after?: string | string[] }>;
}): Promise<ReactElement> {
  const { after } = await searchParams;
  const { teamHandle } = await params;
  const initial = await loadAgents(
    "",
    typeof after === "string" && after.length <= 128 ? after : "",
    teamHandle,
  );
  return (
    <AgentsPage
      basePath={getPublicConfig().publicBasePath}
      key={`${teamHandle}:${initial.type === "list" ? initial.after : ""}`}
      initial={initial}
    />
  );
}
