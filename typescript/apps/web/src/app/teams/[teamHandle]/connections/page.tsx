import { serverSearchValues } from "@chapchoo/web-core/page-state";
import { ConnectionsPage } from "@/features/authentications/ConnectionsPage";
import { loadConnections } from "@/features/authentications/loadConnections";
import type { ReactElement } from "react";
export default async function Page({
  params,
  searchParams,
}: {
  params: Promise<{ teamHandle: string }>;
  searchParams: Promise<{
    after?: string | string[];
    connection?: string | string[];
  }>;
}): Promise<ReactElement> {
  const { teamHandle } = await params;
  const search = await searchParams;
  const input = await loadConnections(teamHandle, serverSearchValues(search));
  return (
    <ConnectionsPage
      key={`${teamHandle}:${input.managed?.id ?? ""}:${input.after ?? ""}`}
      {...input}
    />
  );
}
