import { AppFrame } from "@chapchoo/ui/AppFrame";
import { ApplicationShellContent } from "@chapchoo/ui/ApplicationShell";
import { AppNavigationContext } from "@chapchoo/ui/AppNavigation";
import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { Button, Drawer, Stack } from "@mantine/core";
import { useState } from "react";
import { AgentNavigation } from "./AgentNavigation";
import type { AgentNavigationProps } from "./AgentNavigation";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

function Fixture(): ReactElement {
  const [page, setPage] = useState<string | null>(null);
  const [mobile, setMobile] = useState(false);
  const props: AgentNavigationProps = {
    basePath: "",
    copy: computerCopy("en-US"),
    titleHref: "/teams/design/agents",
    createHref: "/teams/design/new-agent",
    createLabel: "Create Agent",
    selectedHandle: "corgi",
    agentHref: (handle) => `/teams/design/agents/${handle}/chat`,
    items: [
      {
        id: "corgi",
        teamId: "design",
        name: page ? "Bichon" : "Corgi",
        handle: page ? "bichon" : "corgi",
        paletteKey: "sage",
        profileImage: {
          id: "image",
          assetKey: page ? "02-bichon" : "03-corgi",
        },
        computer: {
          operation: "start",
          requestId: "request",
          power: "running",
          lastKnownPower: "running",
          phase: "completed",
          intent: "running",
          management: "connected",
          connector: "connected",
          saved: "verified",
          observedAt: null,
          actions: [],
          forceRemoveAvailable: false,
          startDelayed: false,
          preparationRetryAvailable: false,
        },
      },
    ],
    status: "ready",
    after: page,
    nextAfter: page ? null : "next",
    onPage: setPage,
  };
  const content = <AgentNavigation {...props} />;
  return (
    <AppNavigationContext.Provider
      value={{
        label: "Design team",
        items: [{ key: "team", label: "Team settings", href: "/teams/design" }],
        content,
        header: "Design team",
        trigger: null,
        homeHref: "/",
      }}
    >
      <AppFrame>
        <ApplicationShellContent
          mascotReplayLabel="Replay character animation"
          basePath=""
          title="Agent settings"
          contextLabel="Design team"
          navigation={[]}
        >
          <Stack>
            <Button onClick={() => setMobile(true)}>Open navigation</Button>
          </Stack>
        </ApplicationShellContent>
      </AppFrame>
      <Drawer
        opened={mobile}
        onClose={() => setMobile(false)}
        title="Design team"
      >
        <AgentNavigation {...props} />
      </Drawer>
    </AppNavigationContext.Provider>
  );
}

const meta = {
  title: "Navigation/Team Agents",
  component: Fixture,
  parameters: { layout: "fullscreen" },
} satisfies Meta<typeof Fixture>;
export default meta;
type Story = StoryObj<typeof meta>;
export const TeamPage: Story = {};
