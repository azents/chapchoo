import {
  NumberInput,
  Select,
  Stack,
  Textarea,
  TextInput,
  rem,
} from "@mantine/core";
import { EmailField, PasswordField } from "./fields";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement, ReactNode } from "react";

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

export const EditableControls: Story = {
  render: (): ReactElement => (
    <Stack maw={rem(480)} p="lg">
      <FieldsFixture />
      <TextInput label="Search" type="search" defaultValue="Design notes" />
      <TextInput
        label="Verification code"
        inputMode="numeric"
        defaultValue="123456"
      />
      <TextInput label="Telephone" type="tel" defaultValue="+82 10 1234 5678" />
      <TextInput label="URL" type="url" defaultValue="https://example.com" />
      <NumberInput label="Quantity" defaultValue={3} />
      <Textarea label="Message" defaultValue="A short note." />
      <Select
        label="Workspace"
        data={["Personal", "Team"]}
        defaultValue="Personal"
        searchable
      />
      <label>
        Native select
        <select defaultValue="one">
          <option value="one">One</option>
          <option value="two">Two</option>
        </select>
      </label>
      <div
        role="textbox"
        aria-label="Editable note"
        contentEditable
        suppressContentEditableWarning
      >
        Editable note
      </div>
      <div
        role="textbox"
        aria-label="Bare editable note"
        ref={(element): void => {
          element?.setAttribute("contenteditable", "");
        }}
        suppressContentEditableWarning
      >
        Bare editable note
      </div>
      <div
        role="textbox"
        aria-label="Plain text note"
        contentEditable="plaintext-only"
        suppressContentEditableWarning
      >
        Plain text note
      </div>
    </Stack>
  ),
};
