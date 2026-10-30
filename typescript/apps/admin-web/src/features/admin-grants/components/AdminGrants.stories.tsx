import en from "../../../../messages/en-US.json";
import { AdminGrants } from "./AdminGrants";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

const meta = {
  title: "Admin/Grant inventory",
  component: AdminGrants,
  args: {
    appearance: {
      value: "system",
      label: "Appearance",
      labels: { system: "System", light: "Light", dark: "Dark" },
      onChange: (): void => {},
    },
    kind: "signup",
    basePath: "",
    identityLabel: "admin@example.invalid",
    signInLabel: en.AdminAuth.login,
    state: {
      type: "LOADED",
      items: [
        {
          id: "available",
          email: "new@example.invalid",
          issuedAt: "Oct 30, 2026",
          expiresAt: "Nov 6, 2026",
          status: "active",
        },
        {
          id: "used",
          email: "member@example.invalid",
          issuedAt: "Oct 28, 2026",
          expiresAt: "Nov 4, 2026",
          status: "used",
        },
      ],
      range: "1–2 of 2",
      hasPrevious: false,
      hasNext: false,
    },
    email: "",
    pending: false,
    error: null,
    issued: null,
    copied: false,
    revokeId: null,
    revokeState: { type: "READY" },
    text: (key) => en.AdminGrants[key],
    onEmail: () => {},
    onIssue: () => {},
    onCopy: () => {},
    onDismiss: () => {},
    onRevoke: () => {},
    onCancelRevoke: () => {},
    onConfirmRevoke: () => {},
    onPrevious: () => {},
    onNext: () => {},
    onRefresh: () => {},
    onLogout: () => {},
  },
} satisfies Meta<typeof AdminGrants>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Inventory: Story = {};
export const PasswordResets: Story = { args: { kind: "reset" } };
export const Empty: Story = {
  args: {
    state: {
      type: "LOADED",
      items: [],
      range: "0–0 of 0",
      hasPrevious: false,
      hasNext: false,
    },
  },
};
export const Loading: Story = { args: { state: { type: "LOADING" } } };
export const Failure: Story = {
  args: { state: { type: "ERROR", message: en.Errors.permissionDenied } },
};
export const Issuing: Story = {
  args: { pending: true, email: "new@example.invalid" },
};
export const Issued: Story = {
  args: {
    issued: {
      email: "new@example.invalid",
      url: "https://example.invalid/signup?token=non-redeemable-story-fixture",
    },
  },
};
export const ConfirmRevoke: Story = { args: { revokeId: "available" } };
export const Revoking: Story = {
  args: {
    revokeId: "available",
    revokeState: { type: "PENDING" },
    pending: true,
  },
};
export const RevokeFailed: Story = {
  args: {
    revokeId: "available",
    revokeState: { type: "ERROR", message: en.Errors.generic },
  },
};
