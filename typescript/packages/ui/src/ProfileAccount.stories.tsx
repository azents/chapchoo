import { ProfileAccount } from "./ProfileAccount";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
const meta = {
  title: "Account/Profile account",
  component: ProfileAccount,
  args: {
    title: "Account settings",
    contextLabel: "Profile",
    identityLabel: "companion@example.invalid",
    emailLabel: "Email",
    logoutLabel: "Sign out",
    onLogout: (): void => {},
    mascotReplayLabel: "Replay character animation",
    navigation: [
      {
        key: "account",
        label: "Account settings",
        href: "/profile",
        active: true,
      },
      {
        key: "display",
        label: "Language and display",
        href: "/profile/preferences",
      },
    ],
  },
} satisfies Meta<typeof ProfileAccount>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Account: Story = {};
