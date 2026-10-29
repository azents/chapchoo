import { HelloWorldPage } from "./HelloWorldPage";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

const meta = {
  title: "Home/HelloWorldPage",
  component: HelloWorldPage,
  args: { heading: "Hello, world!" },
} satisfies Meta<typeof HelloWorldPage>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
export const Korean: Story = { args: { heading: "안녕하세요!" } };
