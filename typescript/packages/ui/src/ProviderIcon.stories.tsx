import { ProviderIcon } from "./ProviderIcon";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
const meta = {
  title: "Shared UI/Provider icon",
  component: ProviderIcon,
  args: { kind: "kubernetes", size: 32 },
} satisfies Meta<typeof ProviderIcon>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Kubernetes: Story = {};
export const Docker: Story = { args: { kind: "docker" } };
