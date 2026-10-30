import { ConfirmationDialog } from "./ConfirmationDialog";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

const meta = {
  title: "Shared UI/ConfirmationDialog",
  component: ConfirmationDialog,
  args: {
    opened: true,
    title: "Delete team?",
    message: "The collaborative team and its memberships will be removed.",
    confirmLabel: "Delete team",
    cancelLabel: "Keep team",
    destructive: true,
    onConfirm: (): void => {},
    onCancel: (): void => {},
  },
} satisfies Meta<typeof ConfirmationDialog>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Ready: Story = { args: { state: { type: "READY" } } };
export const Pending: Story = { args: { state: { type: "PENDING" } } };
export const Error: Story = {
  args: {
    state: {
      type: "ERROR",
      message: "The team could not be deleted. Check its current state.",
    },
  },
};
export const Mobile: Story = {
  ...Ready,
  globals: { viewport: { value: "mobile1" } },
};
