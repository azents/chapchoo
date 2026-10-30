import { ToneStory } from "./ToneStory";
import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  title: "Chapchoo/Foundations",
  component: ToneStory,
  parameters: {
    layout: "fullscreen",
    docs: {
      description: {
        component:
          "Selected warm character-led visual tone. Root DESIGN.md is the single design-principle authority. Exact token treatments are reference implementation; no app rollout or sample functionality is approved. iOS typography requires Apple-device verification.",
      },
    },
  },
} satisfies Meta<typeof ToneStory>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Overview: Story = { args: { example: "foundations" } };
export const Components: Story = { args: { example: "components" } };
export const Feedback: Story = { args: { example: "states" } };
export const Mascots: Story = { args: { example: "mascots" } };
export const IOS: Story = { args: { example: "foundations", platform: "ios" } };
