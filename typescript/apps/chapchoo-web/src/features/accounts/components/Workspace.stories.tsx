import messages from "../../../../messages/en-US.json";
import korean from "../../../../messages/ko-KR.json";
import { Workspace } from "./Workspace";
import type { WorkspaceData } from "../types";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

const data: WorkspaceData = {
  identity: "owner@example.com",
  contexts: [
    { id: "personal", name: "Personal", personal: true, role: "OWNER" },
    { id: "design", name: "Design team", personal: false, role: "OWNER" },
  ],
  received: [
    {
      id: "invite",
      teamId: "engineering",
      email: "owner@example.com",
      label: "Engineering",
    },
  ],
  team: null,
};
const team: NonNullable<WorkspaceData["team"]> = {
  id: "design",
  name: "Design team",
  handle: "design",
  personal: false,
  role: "OWNER",
  members: [
    { id: "owner", email: "owner@example.com", role: "OWNER" },
    { id: "manager", email: "manager@example.com", role: "MANAGER" },
    { id: "member", email: "member@example.com", role: "MEMBER" },
  ],
  invitations: [
    {
      id: "pending",
      teamId: "design",
      email: "new-member@example.com",
      label: "Design team",
    },
  ],
};
const meta = {
  title: "Accounts/Workspace",
  component: Workspace,
  args: {
    basePath: "",
    copy: messages.accounts,
    state: { type: "LOADED", data },
    submission: { type: "READY" },
    name: "",
    handle: "",
    inviteEmail: "",
    confirmation: null,
    onName: () => {},
    onHandle: () => {},
    onInviteEmail: () => {},
    onCreate: () => {},
    onRename: () => {},
    onInvite: () => {},
    onInvitation: () => {},
    onAction: () => {},
    onConfirm: () => {},
    onCancel: () => {},
    onRetry: () => {},
    onLogout: () => {},
  },
} satisfies Meta<typeof Workspace>;
export default meta;
type Story = StoryObj<typeof meta>;
export const ContextsAndInvitation: Story = {};
export const EmptyInvitations: Story = {
  args: { state: { type: "LOADED", data: { ...data, received: [] } } },
};
export const Personal: Story = {
  args: {
    state: {
      type: "LOADED",
      data: {
        ...data,
        team: {
          ...team,
          personal: true,
          name: "Personal",
          members: [],
          invitations: [],
        },
      },
    },
  },
};
export const Owner: Story = {
  args: { state: { type: "LOADED", data: { ...data, team } } },
};
export const Manager: Story = {
  args: {
    state: {
      type: "LOADED",
      data: {
        ...data,
        identity: "manager@example.com",
        team: { ...team, role: "MANAGER" },
      },
    },
  },
};
export const Member: Story = {
  args: {
    state: {
      type: "LOADED",
      data: {
        ...data,
        identity: "member@example.com",
        team: { ...team, role: "MEMBER", invitations: [] },
      },
    },
  },
};
export const TransferConfirmation: Story = {
  args: {
    state: { type: "LOADED", data: { ...data, team } },
    confirmation: {
      type: "TRANSFER",
      userId: "member",
      label: "member@example.com",
    },
  },
};
export const DeletePending: Story = {
  args: {
    state: { type: "LOADED", data: { ...data, team } },
    confirmation: { type: "DELETE", label: "Design team" },
    submission: { type: "PENDING" },
  },
};
export const PermissionChanged: Story = {
  args: {
    state: {
      type: "LOADED",
      data: { ...data, team: { ...team, role: "MEMBER" } },
    },
    submission: { type: "ERROR", message: messages.accounts.permissionDenied },
  },
};
export const Loading: Story = { args: { state: { type: "LOADING" } } };
export const Error: Story = {
  args: { state: { type: "ERROR", message: messages.accounts.genericError } },
};
export const Korean: Story = { args: { copy: korean.accounts } };
