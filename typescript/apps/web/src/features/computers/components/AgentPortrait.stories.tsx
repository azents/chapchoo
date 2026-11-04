import { AgentPortrait } from "./AgentPortrait";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
const meta = {
  title: "Computers/Agent portrait",
  component: AgentPortrait,
  args: {
    basePath: "",
    assetKey: "03-corgi",
    paletteKey: "sage",
    compact: false,
  },
} satisfies Meta<typeof AgentPortrait>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Saved: Story = {};
export const Compact: Story = { args: { compact: true } };
export const Rose: Story = { args: { paletteKey: "rose" } };
