import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { AgentCreation } from "./AgentCreation";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

const provider = { id: "provider", name: "Community Kubernetes" };
const initial = {
  teamId: "team",
  identity: "owner@example.invalid",
  providers: { items: [provider], nextAfter: null },
  renderedAt: 1793577600000,
};
const meta = {
  title: "Computers/Agent creation",
  component: AgentCreation,
  args: {
    initial,
    basePath: "",
    access: "authorized",
    copy: computerCopy("en-US"),
    pending: false,
    error: null,
    creationLocked: false,
    name: "Research companion",
    providerId: "provider",
    providers: {
      type: "loaded",
      items: [provider],
      after: null,
      nextAfter: null,
      stale: false,
    },
    onName: (): void => {},
    onProvider: (): void => {},
    onProviderPage: (): void => {},
    onCreate: (): void => {},
    onRefresh: (): void => {},
    onLogout: (): void => {},
  },
} satisfies Meta<typeof AgentCreation>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Ready: Story = {};
export const Empty: Story = {
  args: {
    providers: {
      type: "loaded",
      items: [],
      after: null,
      nextAfter: null,
      stale: false,
    },
    name: "",
    providerId: "",
  },
};
export const ProviderReadFailure: Story = {
  args: { providers: { type: "error" } },
};
export const RetainedCreationForm: Story = {
  args: {
    providers: {
      type: "loaded",
      items: [provider],
      after: null,
      nextAfter: null,
      stale: true,
    },
  },
};
export const UnresolvedCreationWithoutProvider: Story = {
  args: {
    creationLocked: true,
    error: computerCopy("en-US").lostSubmission,
    providers: {
      type: "loaded",
      items: [],
      after: null,
      nextAfter: null,
      stale: false,
    },
  },
};
export const Pending: Story = { args: { pending: true, creationLocked: true } };
export const Japanese: Story = {
  args: {
    copy: computerCopy("ja-JP"),
    providers: Empty.args?.providers,
    name: "",
    providerId: "",
  },
};
export const French: Story = {
  args: {
    copy: computerCopy("fr-FR"),
    providers: Empty.args?.providers,
    name: "",
    providerId: "",
  },
};
export const Korean: Story = {
  args: {
    copy: computerCopy("ko-KR"),
    providers: Empty.args?.providers,
    name: "",
    providerId: "",
  },
};
