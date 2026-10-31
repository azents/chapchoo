import { useState } from "react";
import messages from "../../../../messages/en-US.json";
import korean from "../../../../messages/ko-KR.json";
import { Auth } from "./Auth";
import type { AuthProps } from "../types";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

const ready = {
  type: "LOADED",
  data: { password: true, emailCode: true, signupEmail: true, email: "" },
} satisfies AuthProps["state"];
const meta = {
  title: "Accounts/Authentication",
  component: Auth,
  args: {
    appearance: {
      value: "system",
      label: "Appearance",
      labels: { system: "System", light: "Light", dark: "Dark" },
      onChange: (): void => {},
    },
    basePath: "",
    copy: messages.accounts,
    mode: "LOGIN",
    token: "",
    state: ready,
    submission: { type: "READY" },
    email: "member@example.com",
    password: "",
    code: "",
    challenge: false,
    onEmail: () => {},
    onPassword: () => {},
    onCode: () => {},
    onSubmit: () => {},
    onSendCode: () => {},
    onUsePassword: () => {},
    onRetry: () => {},
  },
} satisfies Meta<typeof Auth>;
export default meta;
type Story = StoryObj<typeof meta>;
export const PasswordLogin: Story = {};
function CharacterLoginFixture(props: AuthProps): ReactElement {
  const [email, setEmail] = useState(props.email);
  const [password, setPassword] = useState(props.password);
  return (
    <Auth
      {...props}
      email={email}
      password={password}
      onEmail={setEmail}
      onPassword={setPassword}
    />
  );
}
export const ServerSelectedCharacter: Story = {
  args: { character: "husky" },
  render: (args): ReactElement => <CharacterLoginFixture {...args} />,
};
export const EmailCode: Story = { args: { challenge: true } };
export const CodeOnly: Story = {
  args: { state: { type: "LOADED", data: { ...ready.data, password: false } } },
};
export const PasswordOnly: Story = {
  args: {
    state: { type: "LOADED", data: { ...ready.data, emailCode: false } },
  },
};
export const LoginUnavailable: Story = {
  args: {
    state: {
      type: "LOADED",
      data: { ...ready.data, emailCode: false, password: false },
    },
  },
};
export const SignupLink: Story = {
  args: {
    mode: "SIGNUP",
    token: "story-proof",
    email: "",
    state: {
      type: "LOADED",
      data: { ...ready.data, email: "m***@example.com" },
    },
  },
};
export const RequestSignup: Story = { args: { mode: "SIGNUP" } };
export const AdministratorLinkRequired: Story = { args: { mode: "RESET" } };
export const ResetPassword: Story = {
  args: {
    mode: "RESET",
    token: "story-proof",
    state: {
      type: "LOADED",
      data: { ...ready.data, email: "m***@example.com" },
    },
  },
};
export const ExpiredLink: Story = {
  args: {
    mode: "SIGNUP",
    token: "story-proof",
    state: { type: "ERROR", message: messages.accounts.invalidProof },
  },
};
export const ResetComplete: Story = {
  args: {
    mode: "RESET",
    token: "story-proof",
    state: ready,
    submission: { type: "DONE", message: messages.accounts.resetDone },
  },
};
export const Pending: Story = { args: { submission: { type: "PENDING" } } };
export const InvalidCredentials: Story = {
  args: {
    submission: {
      type: "ERROR",
      message: messages.accounts.invalidCredentials,
    },
  },
};
export const Korean: Story = {
  args: {
    copy: korean.accounts,
    appearance: {
      value: "system",
      label: "화면 모드",
      labels: { system: "시스템", light: "라이트", dark: "다크" },
      onChange: (): void => {},
    },
  },
};
