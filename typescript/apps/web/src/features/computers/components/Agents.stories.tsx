import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { Agents } from "./Agents";
import type { InitialAgents } from "../types";
import type { AgentView } from "@chapchoo/web-core/computer";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

const agent: AgentView = {
  id: "agent",
  teamId: "team",
  name: "Research companion",
  computer: {
    intent: "running",
    requestId: "request",
    power: "running",
    lastKnownPower: "running",
    phase: "completed",
    management: "connected",
    connector: "connected",
    saved: "verified",
    observedAt: 1793577600000,
    actions: [
      { action: "start", available: false, reason: "requiresStopped" },
      { action: "stop", available: true, reason: null },
      { action: "restart", available: true, reason: null },
      { action: "delete", available: true, reason: null },
    ],
    forceRemoveAvailable: false,
    startDelayed: false,
    preparationRetryAvailable: false,
  },
};
const initial: InitialAgents = {
  type: "list",
  teamId: "team",
  page: { items: [agent], nextAfter: null },
  after: "",
  identity: "owner@example.invalid",
  renderedAt: 1793577600000,
};
const detail: InitialAgents = {
  type: "detail",
  teamId: agent.teamId,
  agent,
  identity: initial.identity,
  renderedAt: initial.renderedAt,
};
const meta = {
  title: "Computers/Agents",
  component: Agents,
  args: {
    access: "authorized",
    initial,
    data: initial,
    basePath: "",
    copy: computerCopy("en-US"),
    stale: false,
    pending: false,
    error: null,
    confirmation: null,
    onAction: () => {},
    onConfirm: () => {},
    onCancel: () => {},
    onRefresh: () => {},
    onLogout: () => {},
    formatTime: () => "Nov 2, 2026, 9:00 AM",
  },
} satisfies Meta<typeof Agents>;
export default meta;
type Story = StoryObj<typeof meta>;
export const AgentList: Story = {};
export const SignedOut: Story = {
  args: { access: "signedOut", initial: detail, data: detail },
};
export const Empty: Story = {
  args: {
    data: {
      ...initial,
      type: "list",
      after: "",
      page: { items: [], nextAfter: null },
    },
  },
};
export const Running: Story = { args: { initial: detail, data: detail } };
export const StopConfirmation: Story = {
  args: { initial: detail, data: detail, confirmation: "stop" },
};
export const StaleUnknown: Story = {
  args: {
    initial: detail,
    stale: true,
    data: {
      ...detail,
      agent: {
        ...agent,
        computer: {
          ...agent.computer,
          power: "unknown",
          phase: "uncertain",
          management: "disconnected",
          connector: "disconnected",
        },
      },
    },
  },
};
export const DelayedStart: Story = {
  args: {
    initial: detail,
    data: {
      ...detail,
      agent: {
        ...agent,
        computer: {
          ...agent.computer,
          power: "unknown",
          phase: "starting",
          startDelayed: true,
        },
      },
    },
  },
};
export const ForceRemoval: Story = {
  args: {
    copy: computerCopy("ko-KR"),
    initial: detail,
    confirmation: "force",
    data: {
      ...detail,
      agent: {
        ...agent,
        computer: {
          ...agent.computer,
          intent: "deleting",
          phase: "deleting",
          forceRemoveAvailable: true,
        },
      },
    },
  },
};
export const RetryPreparation: Story = {
  args: {
    initial: detail,
    data: {
      ...detail,
      agent: {
        ...agent,
        computer: {
          ...agent.computer,
          phase: "blocked",
          power: "unknown",
          preparationRetryAvailable: true,
          actions: agent.computer.actions.map((action) =>
            action.action === "delete"
              ? action
              : { ...action, available: false, reason: "conflicting" },
          ),
        },
      },
    },
  },
};
export const RetryPreparationConfirmation: Story = {
  ...RetryPreparation,
  args: { ...RetryPreparation.args, confirmation: "retryPreparation" },
};
