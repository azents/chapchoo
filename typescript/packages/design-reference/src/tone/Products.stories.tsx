import { ToneStory } from "./ToneStory";
import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  title: "Chapchoo/Products",
  component: ToneStory,
  parameters: {
    layout: "fullscreen",
    docs: {
      description: {
        component:
          "Fictional product compositions for visual design discussion, not Chapchoo feature requirements. Static fixtures; no transport, real saves or agent behavior. Responsive web renders, not native implementation.",
      },
    },
  },
} satisfies Meta<typeof ToneStory>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Library: Story = { args: { example: "library" } };
export const Planner: Story = { args: { example: "planner" } };
export const Settings: Story = { args: { example: "settings" } };
export const ContentLayout: Story = { args: { example: "mobile" } };
export const IOSContent: Story = {
  args: { example: "mobile", platform: "ios" },
};
