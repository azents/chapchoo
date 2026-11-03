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
          nativeScopeKind: "",
          nativeScope: "",
        },
        uid: "original-pvc-uid",
        dockerContainerId: null,
        dockerVolumeCreatedAt: null,
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
            nativeScopeKind: "",
            nativeScope: "",
          },
          uid: "original-pod-uid",
          dockerContainerId: null,
          dockerVolumeCreatedAt: null,
          executionId: "retired-execution",
          presence: "absent",
          observedAt: 1793577600000,
        },
      ],
      nextAfter: null,
    },
  },
};
export const DockerOriginalVolume: Story = {
  args: {
    data: {
      items: [
        {
          identity: {
            kind: "DOCKER_VOLUME",
            namespace: "",
            name: "original-saved-environment",
            nativeScopeKind: "DOCKER_DAEMON",
            nativeScope: `daemon-${"a".repeat(300)}`,
          },
          uid: null,
          dockerContainerId: null,
          dockerVolumeCreatedAt: "2026-11-03T06:45:15Z",
          executionId: null,
          presence: "unknown",
          observedAt: 1793577600000,
        },
        {
          identity: {
            kind: "DOCKER_CONTAINER",
            namespace: "",
            name: "retired-execution",
            nativeScopeKind: "DOCKER_DAEMON",
            nativeScope: `daemon-${"a".repeat(300)}`,
          },
          uid: null,
          dockerContainerId: "b".repeat(64),
          dockerVolumeCreatedAt: null,
          executionId: "retired-execution",
          presence: "absent",
          observedAt: 1793577600000,
        },
      ],
      nextAfter: {
        kind: "DOCKER_VOLUME",
        namespace: "",
        name: "original-saved-environment",
        nativeScopeKind: "DOCKER_DAEMON",
        nativeScope: `daemon-${"a".repeat(300)}`,
      },
    },
  },
};
