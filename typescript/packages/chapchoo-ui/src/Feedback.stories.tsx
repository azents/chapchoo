import { Button, Text } from "@mantine/core";
import { Feedback } from "./Feedback";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

const meta = {
  title: "Shared UI/Feedback",
  component: Feedback,
  decorators: [
    (Story): ReactElement => (
      <div style={{ padding: "var(--mantine-spacing-lg)" }}>
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof Feedback>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Loading: Story = {
  args: { state: { type: "LOADING", message: "Loading teams…" } },
};
export const Error: Story = {
  args: {
    state: {
      type: "ERROR",
      title: "Teams unavailable",
      message: "Your teams could not be loaded. Try again.",
      retryLabel: "Try again",
      onRetry: () => {},
    },
  },
};
export const Empty: Story = {
  args: {
    state: {
      type: "EMPTY",
      title: "No collaborative teams",
      message: "Create a team to work together.",
      action: <Button>Create team</Button>,
    },
  },
};
export const Loaded: Story = {
  args: { state: { type: "LOADED", children: <Text>Research team</Text> } },
};
