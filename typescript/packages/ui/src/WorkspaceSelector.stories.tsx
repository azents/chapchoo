import { WorkspaceSelector } from "./WorkspaceSelector";
import type { WorkspaceChoice } from "./WorkspaceSelector";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

const personal: WorkspaceChoice = {
  key: "personal",
  kind: "personal",
  label: "Your space",
  href: "/teams/my/agents",
  selected: true,
};
const teams: WorkspaceChoice[] = [
  {
    key: "design",
    kind: "team",
    handle: "design",
    label: "Design team",
    href: "/teams/design/agents",
    selected: false,
  },
  {
    key: "research",
    kind: "team",
    handle: "research",
    label: "Research team",
    href: "/teams/research/agents",
    selected: false,
  },
];
const admin: WorkspaceChoice = {
  key: "administration",
  kind: "admin",
  label: "Administration",
  href: "/administration",
  selected: false,
};
const meta = {
  title: "Chrome/Workspace selector",
  component: WorkspaceSelector,
  parameters: { layout: "centered" },
  args: {
    current: personal,
    choices: [personal, ...teams, admin],
    status: "ready",
    mobile: false,
    opened: true,
    copy: {
      choose: "Switch workspace",
      teams: "Teams",
      create: "Create team",
      back: "Back",
      unavailable: "Workspaces are unavailable.",
      retry: "Retry",
    },
    onOpen: (): void => {},
    onClose: (): void => {},
    onCreate: (): void => {},
    onRetry: (): void => {},
    onSelect: (): void => {},
  },
} satisfies Meta<typeof WorkspaceSelector>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Personal: Story = {};
export const Administration: Story = {
  args: {
    current: { ...admin, selected: true },
    choices: [
      { ...personal, selected: false },
      ...teams,
      { ...admin, selected: true },
    ],
  },
};
export const MobileOverlay: Story = { args: { mobile: true } };
export const Unavailable: Story = {
  args: { status: "error", choices: [personal, admin] },
};
