import { Button, Stack } from "@mantine/core";
import { AuthPanel } from "./AuthPanel";
import { EmailField, PasswordField } from "./fields";
import { ErrorFeedback } from "./Feedback";
import { expect, userEvent, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactNode } from "react";

const meta = {
  title: "Shared UI/AuthPanel",
  component: AuthPanel,
  parameters: { layout: "fullscreen" },
  args: {
    title: "Sign in",
    mascotReplayLabel: "Replay character animation",
    description: "Use your account to continue.",
  },
} satisfies Meta<typeof AuthPanel>;

export default meta;
type Story = StoryObj<typeof meta>;

function FormFixture({ disabled = false }: { disabled?: boolean }): ReactNode {
  return (
    <Stack gap="md">
      <EmailField
        label="Email"
        value="member@example.com"
        onValueChange={() => {}}
        disabled={disabled}
      />
      <PasswordField
        label="Password"
        value=""
        onValueChange={() => {}}
        visibilityToggleLabel="Show or hide password"
        disabled={disabled}
      />
      <Button disabled={disabled}>Sign in</Button>
    </Stack>
  );
}

export const Ready: Story = {
  args: { children: <FormFixture /> },
  play: async ({ canvasElement }): Promise<void> => {
    const canvas = within(canvasElement);
    await userEvent.click(canvas.getByLabelText("Email"));
    await userEvent.tab();
    await expect(
      canvas.getByLabelText("Password", { exact: true }),
    ).toHaveFocus();
    await userEvent.tab();
    await expect(
      canvas.getByRole("button", { name: "Show or hide password" }),
    ).toHaveFocus();
  },
};
export const Disabled: Story = {
  args: { children: <FormFixture disabled /> },
};
export const Error: Story = {
  args: {
    children: (
      <Stack>
        <ErrorFeedback
          title="Sign-in unsuccessful"
          message="Check your email and password, then try again."
        />
        <FormFixture />
      </Stack>
    ),
  },
};
export const Mobile: Story = {
  ...Ready,
  globals: { viewport: { value: "mobile1" } },
};
