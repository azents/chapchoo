import { Button, Stack, Text } from "@mantine/core";
import { AppHeader } from "./AppHeader";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

const meta = {
  title: "Shared UI/App header",
  component: AppHeader,
  parameters: { layout: "fullscreen" },
  args: {
    identityLabel: "member@example.com",
    actions: (
      <Button variant="subtle" size="xs">
        Log out
      </Button>
    ),
    appearance: {
      value: "system",
      label: "Appearance",
      labels: { system: "System", light: "Light", dark: "Dark" },
      onChange: (): void => {},
    },
  },
  decorators: [
    (Story): ReactElement => (
      <>
        <Story />
        <Stack p="xl" mih="150dvh">
          <Text>
            Scrollable content keeps the shared appearance entry point visible.
          </Text>
        </Stack>
      </>
    ),
  ],
} satisfies Meta<typeof AppHeader>;
export default meta;
type Story = StoryObj<typeof meta>;
export const SignedIn: Story = {};
export const SignedOut: Story = {
  args: { identityLabel: undefined, actions: undefined },
};
export const LongIdentity: Story = {
  args: {
    identityLabel:
      "long-account-name-with-an-unbroken-identifier-that-must-remain-readable@example.com",
  },
};
