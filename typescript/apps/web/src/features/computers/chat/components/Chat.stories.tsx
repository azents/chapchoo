import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { conversationActivity } from "@chapchoo/web-core/conversation-activity";
import { useState } from "react";
import { fixtureCatalogue } from "@/shared/authentications/connectionFixture";
import { AgentNavigation } from "@/shared/navigation/AgentNavigation";
import { useComputerActions } from "../../containers/useComputerActions";
import { creationCopy } from "../../creationCopy";
import { computerDetailCopy } from "../../detailCopy";
import { chatStatus } from "../chatStatus";
import { useChatFocus } from "../containers/useChatFocus";
import { useFloatingChat } from "../containers/useFloatingChat";
import { useHistoryScroll } from "../containers/useHistoryScroll";
import { activityStatus, chatCopy } from "../copy";
import { Chat } from "./Chat";
import { ConversationHistory } from "./ConversationHistory";
import { conversationReviewRecords } from "./conversationReviewFixture";
import { conversationReviewRecords as presentationRecords } from "./recordPresentationFixture";
import type { AgentsProps, InitialAgents } from "../../types";
import type { ChatCopy } from "../copy";
import type { ConversationControls } from "../types";
import type { AgentView } from "@chapchoo/web-core/computer";
import type { ConversationMessage } from "@chapchoo/web-core/conversation-message-types";
import type {
  ConversationRecord,
  ConversationRegion,
} from "@chapchoo/web-core/conversation-types";
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
    | "nativeFailed"
    | "authenticationRequired"
    | "nativeStarting"
    | "notStarted"
    | "review"
    | "presentation"
    | "pagination"
    | "unread"
    | "idle"
    | "pendingInput"
    | "inputError"
    | "queueError"
    | "queueWaiting"
    | "queuePages"
    | "inputRecorded"
    | "computerStarting"
    | "computerStopping"
    | "computerRestarting"
    | "statusRefreshing"
    | "empty";
  locale: string;
  name: string;
  initialActivityOpened?: boolean;
  underfilled?: boolean;
};
function StaticHistory({
  records,
  copy,
  name,
  focus,
}: {
  records: ConversationRecord[];
  copy: ChatCopy;
  name: string;
  focus: string | null;
}): ReactElement {
  const scroll = useHistoryScroll(focus);
  const controls: ConversationControls = {
    region: {
      header: {
        agentId: "agent",
        conversationId: "conversation",
        state: "ready",
        foreground: "working",
      },
      records,
      renderedAt: 1,
    },
    read: "ready",
    readingOlder: false,
    drafts: {},
    stop: "idle",
    onRead: () => {},
    onOlder: () => {},
    onLatest: scroll.followLatest,
    onHistory: () => {},
    onQuestionOption: () => {},
    onQuestionField: () => {},
    onAnswer: () => {},
  };
  return (
    <ConversationHistory
      controls={controls}
      scroll={scroll}
      copy={copy}
      agentName={name}
    />
  );
}
function Fixture({
  mode,
  locale,
  name,
  initialActivityOpened = false,
  underfilled = false,
}: FixtureProps): ReactElement {
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
      operation:
        mode === "computerRestarting"
          ? "restart"
          : mode === "computerStopping"
            ? "stop"
            : "start",
      intent: mode === "computerStopping" ? "stopped" : "running",
      requestId: "request",
      power: stopped
        ? "stopped"
        : mode === "unknown" ||
            mode.startsWith("computer") ||
            mode === "statusRefreshing"
          ? "unknown"
          : "running",
      lastKnownPower: "running",
      phase:
        mode === "computerStarting"
          ? "accepted"
          : mode === "computerStopping" || mode === "computerRestarting"
            ? "retiring"
            : "completed",
      management: "connected",
      connector:
        mode === "disconnected" || mode === "statusRefreshing"
          ? "disconnected"
          : "connected",
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
  const [input, setInput] = useState(
    mode === "inputError" ? "두 저장소의 작업 현황을 정리해줘." : "",
  );
  const [records, setRecords] = useState<ConversationRecord[]>(
    [
      "pendingInput",
      "inputError",
      "queueError",
      "queueWaiting",
      "inputRecorded",
      "computerStarting",
      "computerStopping",
      "computerRestarting",
      "statusRefreshing",
    ].includes(mode)
      ? [
          fixtureMessage(
            "user",
            "visual-user",
            "안녕, 지금 대화할 수 있어?",
            1n,
          ),
          fixtureMessage(
            "assistant",
            "visual-assistant",
            "네, 준비됐어요. 무엇을 도와드릴까요?",
            2n,
          ),
          ...(mode === "inputRecorded"
            ? [
                fixtureMessage(
                  "user",
                  "visual-recorded",
                  "두 저장소의 작업 현황을 정리해줘.",
                  3n,
                ),
              ]
            : []),
        ]
      : mode === "review" || mode === "presentation" || mode === "pagination"
        ? mode === "presentation"
          ? presentationRecords
          : mode === "pagination"
            ? Array.from({ length: underfilled ? 1 : 30 }, (_, index) =>
                fixtureMessage(
                  "assistant",
                  `page-${index + (underfilled ? 40 : 11)}`,
                  `Message ${index + (underfilled ? 40 : 11)}`,
                  BigInt(index + (underfilled ? 40 : 11)),
                ),
              )
            : conversationReviewRecords
        : mode === "empty" || mode === "idle" || mode === "queuePages"
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
  const [older, setOlder] = useState(mode === "pagination");
  const [pageStart, setPageStart] = useState(underfilled ? 40 : 11);
  const [opened, setOpened] = useState(initialActivityOpened);
  const [historyOpened, setHistoryOpened] = useState(false);
  const [historyFocus, setHistoryFocus] = useState<string | null>(null);
  const [navOpened, setNavOpened] = useState(false);
  const [detailsOpened, setDetailsOpened] = useState(false);
  const [activity, setActivity] = useState<"working" | "idle">(
    mode === "idle" ||
      mode === "inputError" ||
      mode === "queueError" ||
      mode === "queueWaiting"
      ? "idle"
      : "working",
  );
  const [workStop, setWorkStop] = useState<"idle" | "published">("idle");
  const [failedDeleted, setFailedDeleted] = useState(false);
  const [retrySubmitted, setRetrySubmitted] = useState(false);
  const [nativeRecorded, setNativeRecorded] = useState(false);
  const [queueStart, setQueueStart] = useState(6);
  // Static accepted receipts for this UI fixture, distinct from captured records.
  const messages: ConversationMessage[] = (
    [
      "nativeFailed",
      "authenticationRequired",
      "nativeStarting",
      "notStarted",
    ].includes(mode)
      ? []
      : records
  )
    .filter((record) => record.kind === "user" || record.kind === "assistant")
    .map((record) => ({
      id: `receipt-${record.key}`,
      role: record.kind === "user" ? "user" : "agent",
      order: record.anchor.ordinal,
      receivedAt: 1n,
      text: record.content
        .flatMap((part) => (part.type === "text" ? [part.text] : []))
        .join("\n"),
    }));
  const floating = useFloatingChat(input, agent.id, true, messages, {
    source: "conversation/snapshot",
    tail:
      records.filter((record) => record.kind === "request").at(-1)?.anchor ??
      null,
  });
  const copy = chatCopy(locale);
  const c = computerCopy(locale);
  const writable =
    mode === "inputError" ||
    mode === "queueError" ||
    mode === "queueWaiting" ||
    mode === "queuePages" ||
    mode === "pendingInput" ||
    mode === "inputRecorded" ||
    mode === "unread" ||
    mode === "connected" ||
    mode === "review" ||
    mode === "idle" ||
    mode === "empty" ||
    mode === "textPlan" ||
    mode === "nativeFailed" ||
    mode === "notStarted" ||
    mode === "nativeStarting";
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
  const startup = [
    "nativeFailed",
    "authenticationRequired",
    "nativeStarting",
    "notStarted",
  ].includes(mode);
  const startupRegion: ConversationRegion = {
    header: {
      agentId: agent.id,
      conversationId: "conversation",
      state: "notStarted",
      foreground: "unavailable",
      savedAuthenticationId:
        mode === "authenticationRequired" ? void 0 : "saved-auth",
    },
    records: [],
    inputs:
      mode === "nativeFailed" && !failedDeleted
        ? [
            {
              id: retrySubmitted ? "retry-input" : "failed-input",
              requestId: retrySubmitted ? "retry-request" : "request",
              conversationId: "conversation",
              acceptedAt: 1n,
              text: "안녕하세요",
              observation: retrySubmitted ? "accepted" : "failed",
              preparation: retrySubmitted
                ? void 0
                : { type: "failed", reason: "nativeRejected" },
            },
          ]
        : [],
    latestInput:
      mode === "nativeFailed" && !failedDeleted
        ? {
            requestId: retrySubmitted ? "retry-request" : "request",
            observation: retrySubmitted ? "accepted" : "failed",
            preparation: retrySubmitted
              ? void 0
              : { type: "failed", reason: "nativeRejected" },
          }
        : mode === "nativeStarting"
          ? {
              requestId: "request",
              observation: "preparing",
              preparation: { type: "preparing" },
            }
          : void 0,
    renderedAt: initial.renderedAt,
  };
  const displayedActivity = startup
    ? conversationActivity(startupRegion)
    : activity;
  const presentation = chatStatus({
    computer: agent.computer,
    activity: mode === "readerError" ? "unavailable" : displayedActivity,
    foreground: startup ? startupRegion.header.foreground : activity,
    read: mode === "readerError" ? "error" : "ready",
    stale,
    error: product.error,
    computerCopy: c,
    activityLabel: (value) => activityStatus(copy, value),
  });
  function readOlderMessages(): void {
    floating.preserveHistory();
    const start = Math.max(1, pageStart - 10);
    setRecords((current) => [
      ...Array.from({ length: pageStart - start }, (_, index) =>
        fixtureMessage(
          "assistant",
          `page-${index + start}`,
          `Message ${index + start}`,
          BigInt(index + start),
        ),
      ),
      ...current,
    ]);
    setPageStart(start);
    setOlder(start > 1);
  }
  return (
    <>
      <Chat
        messages={{
          region: {
            agentId: agent.id,
            conversationId: "conversation",
            messages,
            through: messages.at(-1)?.order ?? 0n,
            older: older ? BigInt(pageStart) : void 0,
            window: { type: "latest" },
            renderedAt: initial.renderedAt,
          },
          read: mode === "readerError" ? "error" : "ready",
          olderRead: "ready",
          readingOlder: false,
          paginationReset: agent,
          onRead: () => {},
          onOlder: readOlderMessages,
          onLatest: floating.followLatest,
        }}
        recentRecords={{ ...startupRegion, records }}
        recentRead="ready"
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
          region: startup
            ? startupRegion
            : {
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
                inputs:
                  mode === "queuePages"
                    ? Array.from({ length: 30 - queueStart }, (_, offset) => ({
                        id: `queue-page-${queueStart + offset}`,
                        requestId: `queue-page-${queueStart + offset}`,
                        conversationId: "conversation",
                        acceptedAt: BigInt(queueStart + offset),
                        text: `Queued item ${queueStart + offset}`,
                        observation: "failed" as const,
                      }))
                    : (mode === "queueError" || mode === "queueWaiting") &&
                        !failedDeleted
                      ? [
                          {
                            id:
                              retrySubmitted || mode === "queueWaiting"
                                ? "queue-requeued"
                                : "queue-failed",
                            requestId:
                              retrySubmitted || mode === "queueWaiting"
                                ? "requeued"
                                : "failed",
                            conversationId: "conversation",
                            acceptedAt: 1n,
                            text: "두 저장소의 작업 현황을 정리해줘.",
                            observation:
                              retrySubmitted || mode === "queueWaiting"
                                ? "accepted"
                                : "failed",
                          },
                        ]
                      : mode === "pendingInput" && !nativeRecorded
                        ? [
                            {
                              id: "visual-pending-input",
                              requestId: "visual-pending",
                              conversationId: "conversation",
                              acceptedAt: 1n,
                              text: "두 저장소의 작업 현황을 정리해줘.",
                              observation: "accepted",
                            },
                          ]
                        : [],
                older: older
                  ? { ordinal: BigInt(pageStart), index: 0n }
                  : void 0,
                inputOlder:
                  mode === "queuePages" && queueStart > 0
                    ? {
                        acceptedAt: BigInt(queueStart),
                        requestId: `queue-page-${queueStart}`,
                      }
                    : void 0,
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
          stop: workStop,
          inputDeletable: true,
          onRetryInput: () => setRetrySubmitted(true),
          onDeleteInput: () => setFailedDeleted(true),
          drafts: {},
          onRead: () => {},
          onOlder: () => {
            floating.preserveHistory();
            if (mode === "queuePages") {
              setQueueStart(0);
              return;
            }
            const start = Math.max(1, pageStart - 10);
            setRecords((current) => [
              ...Array.from({ length: pageStart - start }, (_, index) =>
                fixtureMessage(
                  "assistant",
                  `page-${index + start}`,
                  `Message ${index + start}`,
                  BigInt(index + start),
                ),
              ),
              ...current,
            ]);
            setPageStart(start);
            setOlder(start > 1);
          },
          onLatest: floating.followLatest,
          onHistory: (record) => {
            setOpened(false);
            setHistoryFocus(record?.key ?? null);
            setHistoryOpened(true);
          },
          onQuestionOption: () => {},
          onQuestionField: () => {},
          onAnswer: () => {},
        }}
        input={input}
        inputError={mode === "inputError" ? copy.enqueueFailed : void 0}
        activity={mode === "readerError" ? "unavailable" : displayedActivity}
        writable={writable}
        stoppable={writable || mode === "sending" || mode === "readerError"}
        sendable={writable && Boolean(input.trim())}
        status={presentation.label}
        statusLoading={presentation.refreshing}
        opened={opened}
        historyOpened={historyOpened}
        historyContent={
          historyOpened ? (
            <StaticHistory
              records={records}
              copy={copy}
              name={name}
              focus={historyFocus}
            />
          ) : null
        }
        onCloseHistory={() => {
          setHistoryOpened(false);
          setOpened(true);
        }}
        navOpened={navOpened}
        onOpen={() => setOpened(true)}
        onClose={() => setOpened(false)}
        onOpenNav={() => setNavOpened(true)}
        onCloseNav={() => setNavOpened(false)}
        onInput={setInput}
        onSend={() => {
          if (mode === "inputError") {
            return;
          }
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
      {mode === "pendingInput" ? (
        <button
          type="button"
          style={{ position: "fixed", top: 0, right: 0, zIndex: 1000 }}
          onClick={() => {
            setRecords((previous) => [
              ...previous,
              fixtureMessage(
                "user",
                "visual-recorded",
                "두 저장소의 작업 현황을 정리해줘.",
                3n,
              ),
            ]);
            setNativeRecorded(true);
          }}
        >
          Record pending input
        </button>
      ) : null}
      {mode === "unread" || mode === "pagination" ? (
        <div style={{ position: "fixed", top: 0, right: 0, zIndex: 1000 }}>
          <button
            type="button"
            onClick={() =>
              setRecords((previous) => [
                ...previous,
                fixtureMessage(
                  "assistant",
                  `remote-${previous.length}`,
                  `Unread message ${previous.length}`,
                  (previous.at(-1)?.anchor.ordinal ?? 0n) + 1n,
                ),
              ])
            }
          >
            Receive message
          </button>
          <button
            type="button"
            onClick={() => {
              const question: ConversationRecord = {
                ...fixtureMessage(
                  "assistant",
                  `question-${records.length}`,
                  "",
                  BigInt(records.length + 1),
                ),
                kind: "request",
                question: { type: "form", title: "New question", fields: [] },
                responder: { token: "fixture-only" },
              };
              setRecords((previous) => [...previous, question]);
            }}
          >
            Receive question
          </button>
          <button
            type="button"
            onClick={() =>
              setRecords((previous) =>
                previous.map((record) =>
                  record.nativeId === "assistant-one"
                    ? {
                        ...record,
                        updatedOrdinal: record.updatedOrdinal + 100n,
                        content: [
                          ...record.content,
                          { type: "text", text: "\nUpdated earlier reply." },
                        ],
                      }
                    : record,
                ),
              )
            }
          >
            Update earlier reply
          </button>
        </div>
      ) : null}
    </>
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
export const ComposerRequestError: Story = { args: { mode: "inputError" } };
export const QueueDeliveryError: Story = { args: { mode: "queueError" } };
export const QueueWaiting: Story = { args: { mode: "queueWaiting" } };
export const QueuePages: Story = { args: { mode: "queuePages" } };
export const UnreadMessages: Story = {
  args: { mode: "unread", locale: "en-US", name: "Corgi" },
};
export const ReviewAllRecords: Story = { args: { mode: "review" } };
export const RecordPresentation: Story = { args: { mode: "presentation" } };
export const UnderfilledPagination: Story = {
  args: {
    mode: "pagination",
    underfilled: true,
    locale: "en-US",
    name: "Corgi",
  },
};
export const AutomaticPagination: Story = {
  args: { mode: "pagination", locale: "en-US", name: "Corgi" },
};
export const ActivitySheet: Story = {
  args: { mode: "review", initialActivityOpened: true },
};
export const Idle: Story = { args: { mode: "idle" } };
export const NoPlanActivity: Story = { args: { mode: "empty" } };
export const ComposerEmpty: Story = {
  args: { mode: "empty", locale: "en-US", name: "Corgi" },
};
export const Stopped: Story = { args: { mode: "stopped" } };
export const FailedStart: Story = { args: { mode: "startError" } };
export const Disconnected: Story = { args: { mode: "disconnected" } };
export const Unknown: Story = { args: { mode: "unknown" } };
export const Stale: Story = { args: { mode: "stale" } };
export const StaleStopped: Story = { args: { mode: "staleStopped" } };
export const SignedOut: Story = { args: { mode: "signedOut" } };
export const Empty: Story = { args: { mode: "empty" } };
export const PendingInput: Story = { args: { mode: "pendingInput" } };
export const InputRecorded: Story = { args: { mode: "inputRecorded" } };
export const ComputerStarting: Story = { args: { mode: "computerStarting" } };
export const ComputerStopping: Story = { args: { mode: "computerStopping" } };
export const ComputerRestarting: Story = {
  args: { mode: "computerRestarting" },
};
export const StatusRefreshing: Story = { args: { mode: "statusRefreshing" } };
export const NativeFailed: Story = { args: { mode: "nativeFailed" } };
export const AuthenticationRequired: Story = {
  args: { mode: "authenticationRequired" },
};
export const NativeStarting: Story = { args: { mode: "nativeStarting" } };
export const NotStarted: Story = { args: { mode: "notStarted" } };
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
