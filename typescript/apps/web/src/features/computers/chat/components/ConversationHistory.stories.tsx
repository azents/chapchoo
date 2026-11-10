import { Box, Button } from "@mantine/core";
import { useState } from "react";
import { useHistoryScroll } from "../containers/useHistoryScroll";
import { chatCopy } from "../copy";
import { ConversationHistory } from "./ConversationHistory";
import type { ConversationControls } from "../types";
import type { ConversationRecord } from "@chapchoo/web-core/conversation-types";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

function message(index: number): ConversationRecord {
  return {
    key: `assistant:${index}`,
    nativeId: String(index),
    kind: "assistant",
    anchor: { ordinal: BigInt(index), index: 0n },
    updatedOrdinal: BigInt(index),
    locations: [],
    content: [
      {
        type: "text",
        text: `Message ${index}: a retained conversation entry for history navigation.`,
      },
    ],
  };
}
function Fixture({ focused }: { focused: boolean }): ReactElement {
  const [records, setRecords] = useState(
    Array.from({ length: 30 }, (_, index) => message(index + 11)),
  );
  const [older, setOlder] = useState(true);
  const scroll = useHistoryScroll(focused ? "assistant:20" : null);
  const copy = chatCopy("en-US");
  const controls: ConversationControls = {
    region: {
      header: { agentId: "agent", state: "ready", foreground: "idle" },
      records,
      renderedAt: 1,
      older: older ? { ordinal: 11n, index: 0n } : void 0,
    },
    read: "ready",
    readingOlder: false,
    inputs: [],
    drafts: {},
    stop: "idle",
    onRead: () => {},
    onOlder: () => {
      scroll.preserveHistory();
      setRecords((current) => [
        ...Array.from({ length: 10 }, (_, index) => message(index + 1)),
        ...current,
      ]);
      setOlder(false);
    },
    onLatest: scroll.followLatest,
    onHistory: () => {},
    onQuestionOption: () => {},
    onQuestionField: () => {},
    onAnswer: () => {},
  };
  return (
    <Box h="100dvh" style={{ display: "flex", flexDirection: "column" }}>
      <Button
        onClick={() =>
          setRecords((current) => [
            ...current,
            message(current.length + (older ? 11 : 1)),
          ])
        }
      >
        Append remote record
      </Button>
      <ConversationHistory
        controls={controls}
        scroll={scroll}
        copy={copy}
        agentName="Corgi"
      />
    </Box>
  );
}
const meta = {
  title: "Chat/Full history",
  component: Fixture,
  args: { focused: false },
  parameters: { layout: "fullscreen" },
} satisfies Meta<typeof Fixture>;
export default meta;
type Story = StoryObj<typeof meta>;
export const FollowingLatest: Story = {};
export const FocusedRecord: Story = { args: { focused: true } };
