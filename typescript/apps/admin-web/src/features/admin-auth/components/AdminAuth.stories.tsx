import { useState } from "react";
import en from "../../../../messages/en-US.json";
import { AdminAuth } from "./AdminAuth";
import type { AdminAuthProps } from "./AdminAuth";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

const meta = {
  title: "Admin/Authentication",
  component: AdminAuth,
  args: {
    appearance: {
      value: "system",
      label: "Appearance",
      labels: { system: "System", light: "Light", dark: "Dark" },
      onChange: (): void => {},
    },
    mode: "login",
    method: "password",
    state: { type: "FORM", pending: false, error: null },
    basePath: "",
    email: "",
    password: "",
    installationProof: "",
    code: "",
    hasBinding: false,
    emailCodeAvailable: true,
    text: (key) => en.AdminAuth[key],
    onEmail: () => {},
    onPassword: () => {},
    onProof: () => {},
    onCode: () => {},
    onMethod: () => {},
    onChangeEmail: () => {},
    onSubmit: () => {},
  },
} satisfies Meta<typeof AdminAuth>;

export default meta;
type Story = StoryObj<typeof meta>;

export const PasswordLogin: Story = {};
function CharacterLoginFixture(props: AdminAuthProps): ReactElement {
  const [email, setEmail] = useState(props.email);
  const [password, setPassword] = useState(props.password);
  return (
    <AdminAuth
      {...props}
      email={email}
      password={password}
      onEmail={setEmail}
      onPassword={setPassword}
    />
  );
}
export const ServerSelectedCharacter: Story = {
  args: { character: "collie" },
  render: (args): ReactElement => <CharacterLoginFixture {...args} />,
};
export const Pending: Story = {
  args: { state: { type: "FORM", pending: true, error: null } },
};
export const Rejected: Story = {
  args: {
    state: { type: "FORM", pending: false, error: en.Errors.permissionDenied },
  },
};
export const InitialSetup: Story = {
  args: { mode: "setup", emailCodeAvailable: false },
};
export const SetupClosed: Story = {
  args: { mode: "setup", state: { type: "CLOSED" } },
};
export const EmailCode: Story = {
  args: { method: "code", hasBinding: true, email: "admin@example.invalid" },
};
