import { Box, Stack } from "@mantine/core";
import { chatCopy } from "../copy";
import { ConversationContent } from "./ConversationBody";
import {
  ConversationRecordDisclosure,
  RecentConversationRecord,
} from "./ConversationRecord";
import { conversationReviewRecords } from "./conversationReviewFixture";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

function Fixture({ preview }: { preview: boolean }): ReactElement {
  const copy = chatCopy("ko-KR");
  return (
    <Box maw="49rem" p="lg" mx="auto">
      <Stack gap="lg">
        {conversationReviewRecords.map((record) =>
          preview ? (
            <RecentConversationRecord
              key={record.key}
              record={record}
              copy={copy}
              agentName="코기"
              onOpen={() => {}}
            />
          ) : (
            <ConversationRecordDisclosure
              key={record.key}
              record={record}
              copy={copy}
              agentName="코기"
            >
              <ConversationContent parts={record.content} />
            </ConversationRecordDisclosure>
          ),
        )}
      </Stack>
    </Box>
  );
}
const meta = {
  title: "Chat/Record row",
  component: Fixture,
  args: { preview: false },
} satisfies Meta<typeof Fixture>;
export default meta;
type Story = StoryObj<typeof meta>;
export const FullHistory: Story = {};
export const RecentPreview: Story = { args: { preview: true } };
