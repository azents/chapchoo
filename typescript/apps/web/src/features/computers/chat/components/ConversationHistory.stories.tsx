import { Box, Button } from "@mantine/core";
import { useState } from "react";
import { useHistoryScroll } from "../containers/useHistoryScroll";
import { chatCopy } from "../copy";
import { ConversationHistory } from "./ConversationHistory";
import type { ConversationControls } from "../types";
import type {
  ConversationRecord,
  ConversationRegion,
} from "@chapchoo/web-core/conversation-types";
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
function Fixture({
  focused,
  underfilled = false,
}: {
  focused: boolean;
  underfilled?: boolean;
}): ReactElement {
  const [records, setRecords] = useState(
    Array.from({ length: underfilled ? 1 : 30 }, (_, index) =>
      message(index + (underfilled ? 40 : 11)),
    ),
  );
  const [older, setOlder] = useState(true);
  const [pageStart, setPageStart] = useState(underfilled ? 40 : 11);
  const region: ConversationRegion = {
    header: {
      agentId: "agent",
      conversationId: "conversation",
      state: "ready",
      foreground: "idle",
    },
    records,
    renderedAt: 1,
    older: older ? { ordinal: BigInt(pageStart), index: 0n } : void 0,
  };
  const scroll = useHistoryScroll(focused ? "assistant:30" : null, region);
  const copy = chatCopy("en-US");
  const controls: ConversationControls = {
    region,
    read: "ready",
    readingOlder: false,
    drafts: {},
    stop: "idle",
    onRead: () => {},
    onOlder: () => {
      scroll.preserveHistory();
      const start = Math.max(1, pageStart - 10);
      setRecords((current) => [
        ...Array.from({ length: pageStart - start }, (_, index) =>
          message(index + start),
        ),
        ...current,
      ]);
      setPageStart(start);
      setOlder(start > 1);
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
            message(Number(current.at(-1)?.anchor.ordinal ?? 0n) + 1),
          ])
        }
      >
        Append remote record
      </Button>
      <Button
        onClick={() =>
          setRecords((current) =>
            current.map((record) => ({
              ...record,
              updatedOrdinal: record.updatedOrdinal + 1n,
            })),
          )
        }
      >
        Update existing record
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
export const UnderfilledPagination: Story = { args: { underfilled: true } };
