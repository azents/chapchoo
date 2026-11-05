import { Text } from "@mantine/core";
import { useMediaQuery } from "@mantine/hooks";
import { useState } from "react";
import { AppFrame } from "./AppFrame";
import { ApplicationShellContent } from "./ApplicationShell";
import { WorkspaceChrome } from "./WorkspaceChrome";
import type { WorkspaceChromeProps } from "./WorkspaceChrome";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

const current = {
  key: "personal",
  kind: "personal" as const,
  label: "Your space",
  href: "/teams/my/agents",
  selected: true,
};
const items = [
  {
    key: "agents",
    label: "Agents",
    href: "/teams/my/agents",
    active: true,
  },
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
    teamAddress: "Team address",
    teamHandleHint:
      "Proposed from the name. You can edit it before creating the team.",
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
  prospectiveAddress: null,
  handleStatus: {
    message: null,
    description: "The handle cannot be changed after creation.",
    available: true,
  },
  error: null,
  choices: [
    {
      key: "profile",
      kind: "profile",
      label: "Profile",
      href: "/profile",
      selected: false,
    },
    current,
    {
      key: "design",
      kind: "team",
      handle: "design",
      label: "Design team",
      href: "/teams/design/agents",
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
  args: {
    creating: true,
    name: "Design team",
    handle: "design",
    prospectiveAddress: "/general/teams/design",
  },
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
export const CreationAvailabilityFailure: Story = {
  args: {
    creating: true,
    name: "Design team",
    handle: "",
    handleStatus: {
      description: "The handle cannot be changed after creation.",
      available: false,
      message: "Could not check availability. Try again.",
    },
  },
};
export const MobileMenu: Story = { args: { mobile: true, drawer: true } };
export const MobilePicker: Story = {
  args: { mobile: true, drawer: true, picker: true },
};
function InteractiveChrome(props: WorkspaceChromeProps): ReactElement {
  const [drawer, setDrawer] = useState(props.drawer);
  const [picker, setPicker] = useState(props.picker);
  const [creating, setCreating] = useState(props.creating);
  return (
    <WorkspaceChrome
      {...props}
      drawer={drawer}
      picker={picker}
      creating={creating}
      openDrawer={() => setDrawer(true)}
      closeDrawer={() => {
        setDrawer(false);
        setPicker(false);
      }}
      openPicker={() => setPicker(true)}
      closePicker={() => setPicker(false)}
      openCreation={() => {
        setPicker(false);
        setCreating(true);
      }}
      closeCreation={() => setCreating(false)}
    />
  );
}
export const DesktopInteractive: Story = {
  render: (args): ReactElement => <InteractiveChrome {...args} />,
};
function ResponsiveChrome(props: WorkspaceChromeProps): ReactElement {
  const mobile = useMediaQuery("(max-width: 48em)");
  return <InteractiveChrome {...props} mobile={mobile} />;
}
export const ResponsiveInteractive: Story = {
  render: (args): ReactElement => <ResponsiveChrome {...args} />,
};
export const MobileInteractive: Story = {
  args: { mobile: true },
  render: (args): ReactElement => <InteractiveChrome {...args} />,
};
export const MobileLongName: Story = {
  args: {
    mobile: true,
    drawer: true,
    current: {
      ...current,
      label:
        "A workspace with a deliberately long name that must not cover Close",
    },
  },
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
    homeTarget: "_blank",
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
const profile = {
  key: "profile",
  kind: "profile" as const,
  label: "Profile",
  href: "/profile",
  selected: true,
};
const profileItems = [
  { key: "account", label: "Account settings", href: "/profile", active: true },
  {
    key: "display",
    label: "Language and display",
    href: "/profile/preferences",
  },
  { key: "invitations", label: "Invitations", href: "/profile/invitations" },
];
export const Profile: Story = {
  render: (args): ReactElement => <InteractiveChrome {...args} />,
  args: {
    current: profile,
    choices: [profile, { ...current, selected: false }],
    items: profileItems,
    children: (
      <AppFrame>
        <ApplicationShellContent
          title="Account settings"
          contextLabel="Profile"
          navigation={profileItems}
          mascotReplayLabel="Replay character animation"
        >
          <Text>companion@example.invalid</Text>
        </ApplicationShellContent>
      </AppFrame>
    ),
  },
};
export const MobileProfile: Story = {
  args: { ...Profile.args, mobile: true, drawer: true },
};
