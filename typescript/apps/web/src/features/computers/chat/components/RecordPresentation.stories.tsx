import { AgentTheme } from "@chapchoo/ui/AgentTheme";
import { Box } from "@mantine/core";
import { chatCopy } from "../copy";
import { ConversationBody } from "./ConversationBody";
import { conversationReviewRecords } from "./recordPresentationFixture";
import type { ConversationControls } from "../types";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

const controls: ConversationControls = {
  region: {
    header: {
      agentId: "agent",
      conversationId: "conversation",
      state: "ready",
      foreground: "idle",
    },
    records: conversationReviewRecords,
    renderedAt: 1,
  },
  read: "ready",
  readingOlder: false,
  inputs: [],
  drafts: {},
  stop: "idle",
  onRead: () => {},
  onOlder: () => {},
  onLatest: () => {},
  onHistory: () => {},
  onQuestionOption: () => {},
  onQuestionField: () => {},
  onAnswer: () => {},
};
function Fixture({
  teal,
  locale,
}: {
  teal: boolean;
  locale: string;
}): ReactElement {
  return (
    <AgentTheme palette={teal ? "teal" : null}>
      <Box maw="49rem" p="lg" mx="auto">
        <ConversationBody
          controls={controls}
          copy={chatCopy(locale)}
          writable={false}
          agentName="Fixture Agent"
          surface="history"
        />
      </Box>
    </AgentTheme>
  );
}
const meta = {
  title: "Chat/Record presentation",
  component: Fixture,
  args: { teal: false, locale: "ko-KR" },
  parameters: { layout: "fullscreen" },
} satisfies Meta<typeof Fixture>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Default: Story = {};
export const ScopedTeal: Story = { args: { teal: true } };
export const English: Story = { args: { locale: "en-US" } };
export const Japanese: Story = { args: { locale: "ja-JP" } };
export const French: Story = { args: { locale: "fr-FR" } };
