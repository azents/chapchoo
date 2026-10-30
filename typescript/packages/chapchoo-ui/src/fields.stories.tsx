import { Stack, rem } from "@mantine/core";
import { EmailField, PasswordField } from "./fields";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactNode } from "react";

function FieldsFixture({
  disabled = false,
  invalid = false,
}: {
  disabled?: boolean;
  invalid?: boolean;
}): ReactNode {
  return (
    <Stack maw={rem(480)} p="lg">
      <EmailField
        label="Email"
        value={invalid ? "not-an-email" : "member@example.com"}
        onValueChange={() => {}}
        disabled={disabled}
        required
        error={invalid ? "Enter a valid email address." : null}
      />
      <PasswordField
        label="Password"
        value=""
        onValueChange={() => {}}
        visibilityToggleLabel="Show or hide password"
        autoComplete="new-password"
        disabled={disabled}
        required
        description="Use 8–64 characters and at least three character classes."
        error={invalid ? "Choose a stronger password." : null}
      />
    </Stack>
  );
}

const meta = {
  title: "Shared UI/Fields",
  component: FieldsFixture,
} satisfies Meta<typeof FieldsFixture>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Ready: Story = {};
export const Disabled: Story = { args: { disabled: true } };
export const Invalid: Story = { args: { invalid: true } };
