import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { Resources } from "./Resources";
import type { InitialResources } from "../resourceTypes";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
const initial: InitialResources = {
  selection: { computerId: "computer-residual" },
  renderedAt: 1793577600000,
  identity: "admin@example.invalid",
  page: {
    items: [
      {
        identity: {
          kind: "PVC",
          namespace: "managed",
          name: "original-saved-environment",
        },
        uid: "original-pvc-uid",
        executionId: null,
        presence: "unknown",
        observedAt: 1793577600000,
      },
    ],
    nextAfter: null,
  },
};
const meta = {
  title: "Computers/Resources",
  component: Resources,
  args: {
    access: "authorized",
    initial,
    data: initial.page,
    basePath: "",
    copy: computerCopy("en-US"),
    stale: false,
    onRefresh: () => {},
    onLogout: () => {},
    formatTime: () => "Nov 2, 2026, 9:00 AM",
  },
} satisfies Meta<typeof Resources>;
export default meta;
type Story = StoryObj<typeof meta>;
export const OriginalUidUnknownPresence: Story = {};
export const SignedOut: Story = { args: { access: "signedOut" } };
export const Empty: Story = { args: { data: { items: [], nextAfter: null } } };
export const ReadFailure: Story = { args: { stale: true } };
export const ConfirmedAbsence: Story = {
  args: {
    data: {
      items: [
        {
          identity: {
            kind: "POD",
            namespace: "managed",
            name: "retired-execution",
          },
          uid: "original-pod-uid",
          executionId: "retired-execution",
          presence: "absent",
          observedAt: 1793577600000,
        },
      ],
      nextAfter: null,
    },
  },
};
