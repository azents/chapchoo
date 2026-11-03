import { AccountSettings } from "./AccountSettings";
import { useState } from "react";
import type { AccountSettingsProps } from "./AccountSettings";
import type { ReactElement } from "react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

const meta = {
  title: "Chrome/Account settings",
  component: AccountSettings,
  args: {
    mobile: false,
    opened: true,
    value: { language: "en", timeZone: "UTC", region: "en-US" },
    selected: {},
    automatic: false,
    disabled: false,
    zones: ["UTC", "Asia/Seoul", "Europe/Paris"],
    copy: {
      title: "Account and settings",
      language: "Language",
      timeZone: "Time zone",
      region: "Regional format",
      follow: "Follow device",
      preview: "Preview",
      unavailable: "Unavailable",
      scope: "Saved to your account.",
      done: "Done",
      close: "Close",
      immediate: "Changes save automatically.",
      saved: "Saved",
      error: "Could not save changes.",
      storageError: "Could not save device settings.",
      retry: "Retry",
    },
    appearance: {
      value: "system",
      label: "Appearance",
      labels: { system: "System", light: "Light", dark: "Dark" },
      onChange: (): void => {},
    },
    identityLabel: "companion@example.invalid",
    profileHref: "/profile",
    profileLabel: "Profile",
    loginHref: "/login",
    loginLabel: "Sign in",
    logoutLabel: "Sign out",
    changed: false,
    error: false,
    storageError: false,
    instant: 1793577600000,
    onChange: (): void => {},
    onRetry: (): void => {},
    onOpen: (): void => {},
    onClose: (): void => {},
    onLogout: (): void => {},
  },
  parameters: { layout: "centered" },
} satisfies Meta<typeof AccountSettings>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Account: Story = {};
export const Visitor: Story = {
  args: {
    identityLabel: undefined,
    automatic: true,
    copy: {
      ...meta.args.copy,
      title: "Settings",
      scope: "Saved on this device.",
    },
  },
};
export const FailedSave: Story = { args: { error: true } };
export const Saved: Story = { args: { changed: true } };
export const Mobile: Story = { args: { mobile: true } };

function InteractiveSettings(props: AccountSettingsProps): ReactElement {
  const [opened, setOpened] = useState(false);
  return (
    <AccountSettings
      {...props}
      opened={opened}
      onOpen={() => setOpened(true)}
      onClose={() => setOpened(false)}
    />
  );
}
export const Interactive: Story = {
  render: (props): ReactElement => <InteractiveSettings {...props} />,
};
export const MobileInteractive: Story = {
  args: { mobile: true },
  render: (props): ReactElement => <InteractiveSettings {...props} />,
};
