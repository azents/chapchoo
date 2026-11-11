import { ConversationContent } from "./ConversationBody";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

const meta = {
  title: "Chat/Ordered content",
  component: ConversationContent,
  args: {
    rawText: true,
    parts: [
      { type: "text", text: "alpha" },
      { type: "text", text: "beta" },
      {
        type: "resource",
        title: "Resource boundary",
        uri: "file:///retained.txt",
        text: "Retained resource body",
      },
      { type: "text", text: "gamma" },
      { type: "diff", path: "change.txt", oldText: "before", newText: "after" },
      { type: "text", text: "delta" },
    ],
  },
} satisfies Meta<typeof ConversationContent>;
export default meta;
type Story = StoryObj<typeof meta>;
export const MixedToolOutput: Story = {};
