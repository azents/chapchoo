import { Box, Button } from "@mantine/core";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { httpLink } from "@trpc/client";
import { useState } from "react";
import superjson from "superjson";
import { trpc } from "@/shared/rpc/client";
import { useConversationReader } from "../containers/useConversationReader";
import { useHistoryScroll } from "../containers/useHistoryScroll";
import { chatCopy } from "../copy";
import { ConversationHistory } from "./ConversationHistory";
import type { ConversationControls } from "../types";
import type { ConversationRegion } from "@chapchoo/web-core/conversation-types";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

const initial: ConversationRegion = {
  header: {
    agentId: "agent",
    conversationId: "conversation",
    state: "ready",
    foreground: "idle",
    source: {
      agentId: "agent",
      conversationId: "conversation",
      snapshotId: "snapshot",
    },
  },
  records: Array.from({ length: 30 }, (_, index) => ({
    key: `assistant:${index + 11}`,
    nativeId: String(index + 11),
    kind: "assistant",
    anchor: { ordinal: BigInt(index + 11), index: 0n },
    updatedOrdinal: BigInt(index + 11),
    locations: [],
    content: [{ type: "text", text: `Message ${index + 11}` }],
  })),
  older: { ordinal: 11n, index: 0n },
  renderedAt: 1,
};
function Reader(): ReactElement {
  const reader = useConversationReader(initial, true, "history");
  const scroll = useHistoryScroll(null, reader.region);
  const controls: ConversationControls = {
    ...reader,
    drafts: {},
    stop: "idle",
    onRead: reader.refresh,
    onOlder: () => {
      scroll.preserveHistory();
      reader.older();
    },
    onLatest: () => {
      reader.latest();
      scroll.followLatest();
    },
    onHistory: () => {},
    onQuestionOption: () => {},
    onQuestionField: () => {},
    onAnswer: () => {},
  };
  return (
    <Box h="100dvh" style={{ display: "flex", flexDirection: "column" }}>
      <Button onClick={reader.refresh}>Poll now</Button>
      <output data-testid="reader-cursors">{`${reader.region.older?.ordinal ?? "end"}/${reader.region.inputOlder?.acceptedAt ?? "end"}/${reader.read}`}</output>
      <output data-testid="reader-page">{reader.olderRead}</output>
      <ConversationHistory
        controls={controls}
        scroll={scroll}
        copy={chatCopy("en-US")}
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
  title: "Chat/Reader",
  component: Fixture,
  parameters: { layout: "fullscreen" },
} satisfies Meta<typeof Fixture>;
export default meta;
type Story = StoryObj<typeof meta>;
export const AutomaticPages: Story = {};
