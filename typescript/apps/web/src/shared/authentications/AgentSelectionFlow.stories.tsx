import { trpc } from "@/shared/rpc/client";
import { RpcProvider } from "@/shared/rpc/provider";
import { AgentAuthenticationPage } from "./AgentAuthenticationPage";
import {
  fixtureAgentAuthentication,
  fixtureCatalogue,
} from "./connectionFixture";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";
function Consumer(): ReactElement {
  const initial = fixtureAgentAuthentication("en").agent;
  const query = trpc.computers.get.useQuery(
    { agentId: initial.id },
    {
      initialData: initial,
      refetchOnMount: false,
      refetchOnReconnect: false,
      refetchOnWindowFocus: false,
      retry: false,
    },
  );
  return (
    <AgentAuthenticationPage
      agent={query.data}
      basePath=""
      snapshots={[fixtureCatalogue]}
      unavailable={query.isError}
    />
  );
}
function Flow(): ReactElement {
  return (
    <RpcProvider
      initialIdentity={{
        type: "LOADED",
        data: {
          id: "fixture-user",
          email: "fixture@example.com",
          isAdmin: false,
          createdAt: { seconds: 0n, nanos: 0 },
          display: { language: "en", timeZone: "UTC", region: "en-US" },
        },
      }}
    >
      <Consumer />
    </RpcProvider>
  );
}
const meta = {
  title: "Authentication/Agent selection flow",
  component: Flow,
} satisfies Meta<typeof Flow>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Current: Story = {};
