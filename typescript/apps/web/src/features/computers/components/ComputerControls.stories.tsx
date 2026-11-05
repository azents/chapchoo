import { useState } from "react";
import agents, { Running } from "./Agents.stories";
import { ComputerControls } from "./ComputerControls";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ComponentProps, ReactElement } from "react";

const detail = Running.args?.data;
if (!detail || detail.type !== "detail") {
  throw new Error("Computer control fixture requires an Agent detail");
}
function Interactive(
  props: ComponentProps<typeof ComputerControls>,
): ReactElement {
  const [opened, setOpened] = useState(false);
  return (
    <ComputerControls
      {...props}
      detailsOpened={opened}
      onOpenDetails={() => setOpened(true)}
      onCloseDetails={() => setOpened(false)}
    />
  );
}
const meta = {
  title: "Computers/Computer controls",
  component: ComputerControls,
  render: Interactive,
  args: {
    agent: detail.agent,
    copy: agents.args.copy,
    detailCopy: agents.args.detailCopy,
    stale: false,
    pending: false,
    detailsOpened: false,
    onOpenDetails: () => {},
    onCloseDetails: () => {},
    onAction: () => {},
    onRefresh: () => {},
    formatTime: () => "Nov 5, 2026, 11:10 AM",
  },
} satisfies Meta<typeof ComputerControls>;
export default meta;
type Story = StoryObj<typeof meta>;
export const On: Story = {};
export const Off: Story = {
  args: {
    agent: {
      ...detail.agent,
      computer: {
        ...detail.agent.computer,
        power: "stopped",
        connector: "disconnected",
        actions: [
          { action: "start", available: true, reason: null },
          { action: "stop", available: false, reason: "requiresRunning" },
          { action: "restart", available: false, reason: "requiresRunning" },
        ],
      },
    },
  },
};
export const Pending: Story = { args: { pending: true } };
export const Stopping: Story = {
  args: {
    agent: {
      ...detail.agent,
      computer: {
        ...detail.agent.computer,
        operation: "stop",
        intent: "stopped",
        phase: "retiring",
        actions: [
          { action: "start", available: true, reason: null },
          { action: "restart", available: false, reason: "conflicting" },
        ],
      },
    },
  },
};
export const VerifyingShutdown: Story = {
  args: {
    agent: {
      ...detail.agent,
      computer: {
        ...detail.agent.computer,
        operation: "stop",
        intent: "stopped",
        phase: "uncertain",
        power: "unknown",
        connector: "disconnected",
        actions: [
          { action: "start", available: true, reason: null },
          { action: "restart", available: false, reason: "conflicting" },
        ],
      },
    },
  },
};
export const StartAfterShutdown: Story = {
  args: {
    agent: {
      ...detail.agent,
      computer: {
        ...detail.agent.computer,
        operation: "stop",
        intent: "running",
        phase: "retiring",
        actions: [
          { action: "start", available: false, reason: "conflicting" },
          { action: "restart", available: false, reason: "conflicting" },
        ],
      },
    },
  },
};
export const Stale: Story = { args: { stale: true } };
export const RestartUnsupported: Story = {
  args: {
    agent: {
      ...detail.agent,
      computer: {
        ...detail.agent.computer,
        actions: [
          { action: "stop", available: true, reason: null },
          { action: "restart", available: false, reason: "unsupported" },
        ],
      },
    },
  },
};
