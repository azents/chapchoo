import { ContextSidebar } from "./ContextSidebar";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
const meta = {
  title: "Chrome/Context sidebar",
  component: ContextSidebar,
  args: {
    label: "Profile",
    header: "Profile",
    items: [
      {
        key: "account",
        label: "Account settings",
        href: "/profile",
        icon: "profile",
        active: true,
      },
      {
        key: "display",
        label: "Language and display",
        href: "/profile/preferences",
        icon: "info",
      },
      {
        key: "invitations",
        label: "Invitations",
        href: "/profile/invitations",
        icon: "mail",
      },
    ],
  },
} satisfies Meta<typeof ContextSidebar>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Profile: Story = {};
export const Team: Story = {
  args: {
    label: "Design team",
    header: "Design team",
    content: "Private Agent choices",
    items: [
      {
        key: "team",
        label: "Team settings",
        href: "/teams/design",
        icon: "team",
      },
    ],
  },
};
