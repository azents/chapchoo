import { Box, Button, Group } from "@mantine/core";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { httpLink } from "@trpc/client";
import { useState } from "react";
import superjson from "superjson";
import { trpc } from "@/shared/rpc/client";
import { useMessageReader } from "../containers/useMessageReader";
import { chatCopy } from "../copy";
import { ConversationBody } from "./ConversationBody";
import type { ConversationControls } from "../types";
import type { ConversationMessageRegion } from "@chapchoo/web-core/conversation-message-types";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

const initial: ConversationMessageRegion = {
  agentId: "agent",
  conversationId: "conversation",
  messages: [
    {
      id: "message-50",
      role: "user",
      order: 50n,
      receivedAt: 50n,
      text: "Accepted SSR receipt",
      sourceInputId: "input-50",
    },
  ],
  through: 50n,
  older: 50n,
  window: { type: "latest" },
  renderedAt: 1,
};
const activity: ConversationControls = {
  region: {
    header: {
      agentId: "agent",
      conversationId: "conversation",
      state: "ready",
      foreground: "idle",
    },
    records: [
      {
        key: "assistant:private",
        nativeId: "private",
        kind: "assistant",
        anchor: { ordinal: 99n, index: 0n },
        updatedOrdinal: 99n,
        content: [
          {
            type: "text",
            text: "Ordinary native output stays in full records",
          },
        ],
        locations: [],
      },
    ],
    renderedAt: 1,
  },
  read: "ready",
  readingOlder: false,
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
function Reader(): ReactElement {
  const [enabled, setEnabled] = useState(true);
  const reader = useMessageReader(initial, enabled);
  return (
    <Box p="lg">
      <Group>
        <Button onClick={reader.refresh}>Poll now</Button>
        <Button onClick={reader.older}>Older</Button>
        <Button onClick={reader.latest}>Latest</Button>
        <Button onClick={() => setEnabled((value) => !value)}>
          {enabled ? "Withdraw access" : "Restore access"}
        </Button>
      </Group>
      <output data-testid="receipt-state">{`${reader.region.conversationId}/${reader.region.through}/${reader.region.next?.order ?? "end"}/${reader.region.older ?? "end"}/${reader.read}/${reader.olderRead}`}</output>
      <ConversationBody
        controls={activity}
        messages={{
          ...reader,
          onRead: reader.refresh,
          onOlder: reader.older,
          onLatest: reader.latest,
        }}
        copy={chatCopy("en")}
        writable={false}
        agentName="Corgi"
      />
    </Box>
  );
}
function Fixture(): ReactElement {
  const [queryClient] = useState(() => new QueryClient());
  const [client] = useState(() =>
    trpc.createClient({
      links: [
        httpLink({
          url: "/api/trpc",
          transformer: superjson,
          methodOverride: "POST",
        }),
      ],
    }),
  );
  return (
    <trpc.Provider client={client} queryClient={queryClient}>
      <QueryClientProvider client={queryClient}>
        <Reader />
      </QueryClientProvider>
    </trpc.Provider>
  );
}
const meta = {
  title: "Chat/Message reader",
  component: Fixture,
  parameters: { layout: "fullscreen" },
} satisfies Meta<typeof Fixture>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Continuation: Story = {};
