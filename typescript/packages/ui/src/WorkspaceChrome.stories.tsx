import { Text } from "@mantine/core";
import { AppFrame } from "./AppFrame";
import { ApplicationShellContent } from "./ApplicationShell";
import { WorkspaceChrome } from "./WorkspaceChrome";
import type { WorkspaceChromeProps } from "./WorkspaceChrome";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

const current = {
  key: "personal",
  kind: "personal" as const,
  label: "Your space",
  href: "/agents?team=personal",
  selected: true,
};
const items = [
  {
    key: "agents",
    label: "Agents",
    href: "/agents?team=personal",
    active: true,
  },
  { key: "invitations", label: "Invitations", href: "/invitations" },
];
const base: WorkspaceChromeProps = {
  children: (
    <AppFrame>
      <ApplicationShellContent
        mascotReplayLabel="Replay character animation"
        contextLabel="Your space"
        title="Agents"
        navigation={items}
      >
        <Text>No Agents yet.</Text>
      </ApplicationShellContent>
    </AppFrame>
  ),
  identity: true,
  copy: {
    chooseContext: "Switch workspace",
    teams: "Teams",
    createTeam: "Create team",
    back: "Back",
    unavailable: "Could not load workspaces.",
    retry: "Retry",
    navigation: "Open navigation",
    close: "Close",
    teamName: "Team name",
    teamHandle: "Handle",
    cancel: "Cancel",
    create: "Create",
  },
  basePath: "",
  homeHref: "/",
  mobile: false,
  drawer: false,
  picker: false,
  creating: false,
  name: "",
  handle: "",
  error: null,
  choices: [
    current,
    {
      key: "design",
      kind: "team",
      label: "Design team",
      href: "/agents?team=design",
      selected: false,
    },
  ],
  current,
  items,
  status: "ready",
  busy: false,
  mascotReplayLabel: "Replay character animation",
  openDrawer: (): void => {},
  closeDrawer: (): void => {},
  openPicker: (): void => {},
  closePicker: (): void => {},
  openCreation: (): void => {},
  closeCreation: (): void => {},
  onName: (): void => {},
  onHandle: (): void => {},
  onSubmit: (): void => {},
  retry: (): void => {},
  select: (): void => {},
};
const meta = {
  title: "Navigation/Workspace chrome",
  component: WorkspaceChrome,
  args: base,
  parameters: { layout: "fullscreen" },
} satisfies Meta<typeof WorkspaceChrome>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Personal: Story = {};
export const Selector: Story = { args: { picker: true } };
export const Creation: Story = {
  args: { creating: true, name: "Design team", handle: "design" },
};
export const CreationError: Story = {
  args: {
    creating: true,
    name: "Design team",
    handle: "design",
    error: "Check the team list before submitting again.",
  },
};
export const CreationPending: Story = {
  args: { creating: true, name: "Design team", handle: "design", busy: true },
};
export const MobileMenu: Story = { args: { mobile: true, drawer: true } };
export const MobilePicker: Story = {
  args: { mobile: true, drawer: true, picker: true },
};
export const Unavailable: Story = {
  args: { status: "error", picker: true, choices: [], current: null },
};
const administration = {
  key: "administration",
  kind: "admin" as const,
  label: "Administration",
  href: "/admin/",
  selected: true,
};
const adminItems = [
  { key: "signup", label: "Signup links", href: "/admin/", active: true },
  {
    key: "reset",
    label: "Password reset links",
    href: "/admin/password-resets",
  },
  {
    key: "computers",
    label: "Computer Providers",
    href: "/admin/computer-providers",
  },
  { key: "settings", label: "Server settings", href: "/admin/settings" },
];
export const Administration: Story = {
  args: {
    basePath: "/admin",
    homeHref: "/",
    environment: "admin",
    current: administration,
    choices: [{ ...current, selected: false }, administration],
    items: adminItems,
    picker: true,
    children: (
      <AppFrame>
        <ApplicationShellContent
          mascotReplayLabel="Replay character animation"
          contextLabel="Administration"
          title="Signup links"
          navigation={adminItems}
        >
          <Text>No links yet.</Text>
        </ApplicationShellContent>
      </AppFrame>
    ),
  },
};
export const MobileAdministration: Story = {
  args: { ...Administration.args, mobile: true, drawer: true },
};
