import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { useState } from "react";
import { fixtureCatalogue } from "@/shared/authentications/connectionFixture";
import { AgentNavigation } from "@/shared/navigation/AgentNavigation";
import { useComputerActions } from "../../containers/useComputerActions";
import { creationCopy } from "../../creationCopy";
import { computerDetailCopy } from "../../detailCopy";
import { useChatFocus } from "../containers/useChatFocus";
import { useFloatingChat } from "../containers/useFloatingChat";
import { chatCopy } from "../copy";
import { Chat } from "./Chat";
import type { AgentsProps, InitialAgents } from "../../types";
import type { AgentView } from "@chapchoo/web-core/computer";
import type { ConversationRecord } from "@chapchoo/web-core/conversation-types";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";
function fixtureMessage(
  kind: "user" | "assistant",
  nativeId: string,
  text: string,
  ordinal: bigint,
): ConversationRecord {
  return {
    key: `${kind}:${nativeId}`,
    kind,
    nativeId,
    anchor: { ordinal, index: 0n },
    updatedOrdinal: ordinal,
    content: [{ type: "text", text }],
    locations: [],
  };
}

type FixtureProps = {
  mode:
    | "connected"
    | "sending"
    | "readerError"
    | "textPlan"
    | "stopped"
    | "disconnected"
    | "unknown"
    | "stale"
    | "staleStopped"
    | "signedOut"
    | "startError"
    | "empty";
  locale: string;
  name: string;
};
function Fixture({ mode, locale, name }: FixtureProps): ReactElement {
  const stopped =
    mode === "stopped" || mode === "startError" || mode === "staleStopped";
  const stale = mode === "stale" || mode === "staleStopped";
  const agent: AgentView = {
    id: "agent",
    teamId: "team",
    name,
    handle: "corgi",
    profileImage: { id: "corgi", assetKey: "03-corgi" },
    paletteKey: "teal",
    authentication: null,
    computer: {
      operation: "start",
      intent: "running",
      requestId: "request",
      power: stopped ? "stopped" : mode === "unknown" ? "unknown" : "running",
      lastKnownPower: "running",
      phase: "completed",
      management: "connected",
      connector: mode === "disconnected" ? "disconnected" : "connected",
      saved: "verified",
      observedAt: 1793854800000,
      actions: [
        {
          action: "start",
          available: stopped,
          reason: stopped ? null : "requiresStopped",
        },
        {
          action: "stop",
          available: !stopped,
          reason: stopped ? "requiresRunning" : null,
        },
        { action: "restart", available: !stopped, reason: null },
      ],
      forceRemoveAvailable: false,
      startDelayed: false,
      preparationRetryAvailable: false,
    },
  };
  const initial: InitialAgents = {
    type: "detail",
    authenticationSnapshots: [fixtureCatalogue],
    agent,
    renderedAt: 1793854800000,
    identity: "owner@example.invalid",
    teamId: "team",
    teamHandle: "my",
  };
  const [input, setInput] = useState("");
  const [records, setRecords] = useState<ConversationRecord[]>(
    mode === "empty"
      ? []
      : [
          fixtureMessage(
            "user",
            "user-one",
            "  Plan a quiet afternoon in Seoul.  ",
            1n,
          ),
          fixtureMessage(
            "assistant",
            "assistant-one",
            "Let's leave some room to explore.\n\n**First**, a walk through the park.\n\n" +
              "A complete retained paragraph for the reading-position fixture.\n\n".repeat(
                24,
              ),
            2n,
          ),
          fixtureMessage(
            "assistant",
            "assistant-two",
            "I am checking the opening hours.",
            3n,
          ),
        ],
  );
  const [opened, setOpened] = useState(false);
  const [navOpened, setNavOpened] = useState(false);
  const [detailsOpened, setDetailsOpened] = useState(false);
  const [activity, setActivity] = useState<"working" | "idle">("working");
  const [workStop, setWorkStop] = useState<"idle" | "published">("idle");
  const floating = useFloatingChat(input, agent.id);
  const copy = chatCopy(locale);
  const c = computerCopy(locale);
  const writable =
    mode === "connected" || mode === "empty" || mode === "textPlan";
  const actions = useComputerActions({
    pending: false,
    submit: () => {},
    reset: () => {},
  });
  const product: AgentsProps = {
    authenticationControl: null,
    initial,
    data: initial,
    basePath: "",
    access: mode === "signedOut" ? "signedOut" : "authorized",
    copy: c,
    profileCopy: creationCopy(locale),
    detailCopy: computerDetailCopy(locale),
    detailsOpened,
    onOpenDetails: () => setDetailsOpened(true),
    onCloseDetails: () => setDetailsOpened(false),
    mobile: false,
    editor: { type: "closed" },
    images: { items: [], nextAfter: null },
    imageAfter: null,
    imagesUnavailable: false,
    onEditProfile: () => {},
    onProfileName: () => {},
    onProfileImage: () => {},
    onProfilePalette: () => {},
    onRandomProfilePalette: () => {},
    onImagePage: () => {},
    onSaveProfile: () => {},
    onCancelProfile: () => {},
    stale,
    pending: false,
    error: mode === "startError" ? c.lostSubmission : null,
    confirmation: actions.confirmation?.action ?? null,
    onAction: (action) => actions.select(action, agent.computer.requestId),
    onConfirm: actions.confirm,
    onCancel: actions.cancel,
    onRefresh: () => {},
    onLogout: () => {},
    formatTime: () => "13:10",
  };
  const controlled = useChatFocus(product);
  return (
    <Chat
      product={controlled.product}
      focus={controlled.focus}
      agent={agent}
      copy={copy}
      navigation={null}
      accountControls={null}
      supplyControl={null}
      secondaryOverlay={false}
      agentNavigation={
        <AgentNavigation
          basePath=""
          copy={c}
          titleHref="/teams/my/agents"
          createHref="/teams/my/new-agent"
          createLabel={product.profileCopy.newAgent}
          allAgentsLabel="All Agents"
          selectedHandle="corgi"
          agentHref={(handle) => `/teams/my/agents/${handle}/chat`}
          status="ready"
          items={[
            agent,
            {
              ...agent,
              id: "second",
              handle: "bichon",
              name: locale.startsWith("ko") ? "비숑" : "Bichon",
              profileImage: { id: "bichon", assetKey: "02-bichon" },
              paletteKey: "lilac",
            },
          ]}
        />
      }
      conversation={{
        region: {
          header: {
            agentId: agent.id,
            conversationId: "conversation",
            state: "ready",
            source: {
              agentId: agent.id,
              conversationId: "conversation",
              snapshotId: "snapshot",
            },
            native: { snapshotId: "snapshot", nativeSessionId: "native" },
            foreground: activity,
            workId: activity === "working" ? "work" : void 0,
          },
          records,
          latestPlan:
            mode === "textPlan"
              ? {
                  key: "plan:text-plan",
                  nativeId: "text-plan",
                  kind: "plan",
                  anchor: { ordinal: 4n, index: 0n },
                  updatedOrdinal: 4n,
                  content: [],
                  locations: [],
                  plan: {
                    type: "text",
                    text: "**First**, inspect the current implementation.",
                  },
                }
              : null,
          renderedAt: initial.renderedAt,
        },
        read: mode === "readerError" ? "error" : "ready",
        readingOlder: false,
        inputs: [],
        stop: workStop,
        drafts: {},
        onRead: () => {},
        onOlder: () => {},
        onLatest: floating.followLatest,
        onHistory: () => {},
        onQuestionOption: () => {},
        onQuestionField: () => {},
        onAnswer: () => {},
      }}
      input={input}
      activity={mode === "readerError" ? "unavailable" : activity}
      writable={writable}
      stoppable={writable || mode === "sending" || mode === "readerError"}
      sendable={writable && Boolean(input.trim())}
      status={
        product.error ??
        (stale
          ? c.stale
          : mode === "stopped"
            ? c.stopped
            : mode === "disconnected"
              ? c.disconnected
              : mode === "unknown"
                ? c.unknown
                : activity === "working"
                  ? copy.working
                  : copy.idle)
      }
      opened={opened}
      navOpened={navOpened}
      onOpen={() => setOpened(true)}
      onClose={() => setOpened(false)}
      onOpenNav={() => setNavOpened(true)}
      onCloseNav={() => setNavOpened(false)}
      onInput={setInput}
      onSend={() => {
        if (!writable || !input.trim()) {
          return;
        }
        floating.followLatest();
        setRecords((previous) => [
          ...previous,
          fixtureMessage(
            "user",
            `user-${previous.length}`,
            input,
            BigInt(previous.length + 1),
          ),
        ]);
        setInput("");
        setActivity("idle");
      }}
      onStop={() => setWorkStop("published")}
      floating={floating}
    />
  );
}
const meta = {
  title: "Chat/Conversation",
  component: Fixture,
  parameters: { layout: "fullscreen" },
  args: { mode: "connected", locale: "ko-KR", name: "코기" },
} satisfies Meta<typeof Fixture>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Connected: Story = {};
export const NoPlanActivity: Story = { args: { mode: "empty" } };
export const Stopped: Story = { args: { mode: "stopped" } };
export const FailedStart: Story = { args: { mode: "startError" } };
export const Disconnected: Story = { args: { mode: "disconnected" } };
export const Unknown: Story = { args: { mode: "unknown" } };
export const Stale: Story = { args: { mode: "stale" } };
export const StaleStopped: Story = { args: { mode: "staleStopped" } };
export const SignedOut: Story = { args: { mode: "signedOut" } };
export const Empty: Story = { args: { mode: "empty" } };
export const SendingWithActiveWork: Story = {
  args: { mode: "sending", locale: "en-US", name: "Corgi" },
};
export const ReaderErrorWithActiveWork: Story = {
  args: { mode: "readerError", locale: "en-US", name: "Corgi" },
};
export const TextPlan: Story = {
  args: { mode: "textPlan", locale: "en-US", name: "Corgi" },
};
export const English: Story = { args: { locale: "en-US", name: "Corgi" } };
export const Japanese: Story = { args: { locale: "ja-JP", name: "コーギー" } };
export const French: Story = {
  args: { locale: "fr-FR", name: "Compagnon de recherche au nom très long" },
};
