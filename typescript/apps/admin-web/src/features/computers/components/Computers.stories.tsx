import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { Computers } from "./Computers";
import type { InitialComputers } from "../types";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
const initial: InitialComputers = {
  type: "providers",
  page: {
    items: [
      {
        id: "provider",
        kind: "kubernetes",
        name: "Community Kubernetes",
        management: "connected",
        canCreate: true,
      },
      {
        id: "incapable",
        kind: "docker",
        name: "Existing Computers only",
        management: "connected",
        canCreate: false,
      },
    ],
    nextAfter: null,
  },
  after: "",
  renderedAt: 1793577600000,
  identity: "admin@example.invalid",
};
const inventory: InitialComputers = {
  ...initial,
  type: "computers",
  providerId: "provider",
  page: {
    items: [
      {
        id: "computer-retained",
        intent: "deleting",
        power: "unknown",
        lastKnownPower: "running",
        phase: "uncertain",
        management: "disconnected",
        connector: "disconnected",
        saved: "unknown",
        observedAt: 1793577600000,
        residual: true,
        cleanupCompletedAt: null,
      },
    ],
    nextAfter: null,
  },
};
const meta = {
  title: "Computers/OperatorInventory",
  component: Computers,
  args: {
    access: "authorized",
    initial,
    data: initial,
    basePath: "",
    copy: computerCopy("en-US"),
    stale: false,
    formatTime: () => "Nov 2, 2026, 9:00 AM",
    onRefresh: () => {},
    onLogout: () => {},
  },
} satisfies Meta<typeof Computers>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Providers: Story = {};
export const SignedOut: Story = {
  args: { access: "signedOut", initial: inventory, data: inventory },
};
export const ResidualComputer: Story = {
  args: { initial: inventory, data: inventory },
};
export const Stale: Story = {
  args: { initial: inventory, data: inventory, stale: true },
};
export const EmptyInventory: Story = {
  args: {
    data: {
      ...inventory,
      type: "computers",
      providerId: "provider",
      page: { items: [], nextAfter: null },
    },
  },
};
export const ConfirmedCleanup: Story = {
  args: {
    copy: computerCopy("ko-KR"),
    initial: inventory,
    data: {
      ...inventory,
      type: "computers",
      providerId: "provider",
      page: {
        items: [
          {
            intent: "deleting",
            id: "completed-computer",
            power: "stopped",
            lastKnownPower: "stopped",
            phase: "completed",
            management: "connected",
            connector: "disconnected",
            saved: "missing",
            observedAt: 1793577600000,
            residual: false,
            cleanupCompletedAt: 1793577600000,
          },
        ],
        nextAfter: null,
      },
    },
  },
};
