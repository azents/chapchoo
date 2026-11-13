"use client";
import { HeaderControlsContext } from "@chapchoo/ui/AppHeader";
import { AppNavigationContext } from "@chapchoo/ui/AppNavigation";
import { conversationActivity } from "@chapchoo/web-core/conversation-activity";
import {
  initialQuestionDraft,
  reviewQuestionAnswer,
} from "@chapchoo/web-core/conversation-form";
import { useDisplay } from "@chapchoo/web-core/display-context";
import { TRPCClientError } from "@trpc/client";
import { useLocale } from "next-intl";
import { createElement, useContext, useRef, useState } from "react";
import { trpc } from "@/shared/rpc/client";
import { useAgentsContainer } from "../../containers/useAgentsContainer";
import { chatStatus } from "../chatStatus";
import { ConversationHistoryPage } from "../ConversationHistoryPage";
import { activityStatus, chatCopy } from "../copy";
import { SupplyPage } from "../SupplyPage";
import { useChatFocus } from "./useChatFocus";
import { useConversationReader } from "./useConversationReader";
import { useFloatingChat } from "./useFloatingChat";
import { useMessageReader } from "./useMessageReader";
import type { ChatInput, ChatProps, ConversationControls } from "../types";
import type { QuestionDraft } from "@chapchoo/web-core/conversation-form";
import type {
  ConversationAnswer,
  ConversationRecord,
  ConversationRegion,
  RetainedInputView,
} from "@chapchoo/web-core/conversation-types";

export function useChatContainer(props: ChatInput): ChatProps {
  const baseProduct = useAgentsContainer(props);
  const navigation = useContext(AppNavigationContext);
  const accountControls = useContext(HeaderControlsContext);
  const display = useDisplay();
  const secondaryOverlay = display.opened || Boolean(navigation?.overlayActive);
  const controlled = useChatFocus(baseProduct, secondaryOverlay);
  const product = controlled.product;
  const copy = chatCopy(useLocale());
  const [input, setInput] = useState("");
  const [opened, setOpened] = useState(false);
  const [history, setHistory] = useState<{
    initial: ConversationRegion;
    key: string | null;
    olderOnOpen: boolean;
    initialFetch: boolean;
  } | null>(null);
  const [navOpened, setNavOpened] = useState(false);
  const [inputError, setInputError] = useState(false);
  const [drafts, setDrafts] = useState<Partial<Record<string, QuestionDraft>>>(
    {},
  );
  const [stop, setStop] = useState<{
    workId: string;
    requestId: string;
    conversationId: string;
    snapshotId: string;
    state: ConversationControls["stop"];
  } | null>(null);
  const inputBusy = useRef(false);
  const deletionBusy = useRef(false);
  const [inputAction, setInputAction] =
    useState<ConversationControls["inputAction"]>();
  const answerBusy = useRef(new Set<string>());
  const stopBusy = useRef(false);
  const utils = trpc.useUtils();
  const sendMutation = trpc.conversations.send.useMutation({ retry: false });
  const retryMutation = trpc.conversations.retryInput.useMutation({
    retry: false,
  });
  const deleteMutation = trpc.conversations.deleteInput.useMutation({
    retry: false,
  });
  const answerMutation = trpc.conversations.answer.useMutation({
    retry: false,
  });
  const stopMutation = trpc.conversations.stop.useMutation({ retry: false });
  const reader = useConversationReader(
    props.initialConversation,
    product.access === "authorized",
    "activity",
    stop ?? void 0,
  );
  const region = reader.region;
  const messageReader = useMessageReader(
    props.initialMessages,
    product.access === "authorized",
  );
  // Full records have their own lazy read owner; activity is not a history seed.
  const recentInitial = useRef<ConversationRegion>({
    ...props.initialConversation,
    records: [],
    older: void 0,
    inputOlder: void 0,
  });
  const recentReader = useConversationReader(
    recentInitial.current,
    product.access === "authorized" && opened,
    "history",
    void 0,
    true,
  );
  const floating = useFloatingChat(
    input,
    messageReader.region.conversationId ?? "",
    product.access === "authorized",
    messageReader.region.messages,
    {
      source: `${region.header.conversationId}:${region.header.source?.snapshotId ?? ""}`,
      tail: region.records.at(-1)?.anchor ?? null,
    },
  );
  if (product.data.type !== "detail") {
    throw new Error("Chat requires an admitted Agent detail snapshot");
  }
  const agent = product.data.agent;
  const computer = agent.computer;
  const effectAvailable =
    product.access === "authorized" &&
    !product.stale &&
    !product.pending &&
    computer.power === "running" &&
    computer.connector === "connected" &&
    computer.phase === "completed";
  const activity = conversationActivity(region);
  const writable =
    effectAvailable &&
    reader.read === "ready" &&
    messageReader.read === "ready" &&
    region.header.conversationId === messageReader.region.conversationId &&
    region.header.state !== "loading" &&
    activity !== "authenticationRequired";
  const presentation = chatStatus({
    computer,
    activity,
    foreground: region.header.foreground,
    read: reader.read,
    stale: product.stale,
    error: product.error,
    computerCopy: product.copy,
    activityLabel: (value) => activityStatus(copy, value),
  });
  const status = presentation.label;
  const scope = JSON.stringify([
    region.header.agentId,
    region.header.conversationId,
    region.header.source?.snapshotId,
  ]);
  const draftKey = (record: ConversationRecord): string =>
    `${scope}:${record.key}`;
  function currentDraft(record: ConversationRecord): QuestionDraft | null {
    return (
      drafts[draftKey(record)] ??
      (record.question ? initialQuestionDraft(record.question) : null)
    );
  }
  function changeDraft(
    record: ConversationRecord,
    change: (draft: QuestionDraft) => QuestionDraft,
  ): void {
    const key = draftKey(record);
    if (
      !record.question ||
      !writable ||
      !record.responder ||
      answerBusy.current.has(key) ||
      record.answer
    ) {
      return;
    }
    const question = record.question;
    setDrafts((all) => {
      const current = all[key] ?? initialQuestionDraft(question);
      return current.state === "editing" || current.state === "invalid"
        ? { ...all, [key]: { ...change(current), state: "editing" } }
        : all;
    });
  }
  async function send(): Promise<void> {
    const text = input;
    if (!writable || inputBusy.current || !text.trim()) {
      return;
    }
    inputBusy.current = true;
    setInputError(false);
    const requestId = crypto.randomUUID();
    const target = {
      agentId: agent.id,
      conversationId: region.header.conversationId,
      snapshotId: region.header.source?.snapshotId,
      requestId,
      text,
    };
    try {
      await sendMutation.mutateAsync(target);
      setInput((current) => (current === text ? "" : current));
      floating.followLatest();
    } catch {
      // Keep the draft. Only the server queue can introduce a pending bubble,
      // including when acceptance succeeded but its response was lost.
      setInputError(true);
    } finally {
      inputBusy.current = false;
    }
    await refreshAfterEffect();
  }
  function currentFailedInput(
    value: RetainedInputView,
  ): RetainedInputView | null {
    return (
      region.inputs?.find(
        (item) =>
          item.id === value.id &&
          item.requestId === value.requestId &&
          item.conversationId === region.header.conversationId &&
          item.observation === "failed",
      ) ?? null
    );
  }
  async function retryInput(value: RetainedInputView): Promise<void> {
    const current = currentFailedInput(value);
    if (!current || deletionBusy.current || !writable) {
      return;
    }
    deletionBusy.current = true;
    const snapshotId = region.header.source?.snapshotId;
    setInputAction({ requestId: current.requestId, state: "retrying" });
    try {
      await retryMutation.mutateAsync({
        source: {
          agentId: agent.id,
          conversationId: current.conversationId,
          snapshotId,
        },
        originalInputId: current.id,
        originalRequestId: current.requestId,
        requestId: crypto.randomUUID(),
      });
      reader.removeInput(current.id, current.conversationId, snapshotId);
      floating.followLatest();
    } catch {
      // Atomic replacement keeps the original failed item if admission fails.
      // A lost response is resolved only by queried queue rows, never replay.
    } finally {
      deletionBusy.current = false;
      setInputAction(void 0);
    }
    await refreshAfterEffect();
  }
  async function deleteInput(value: RetainedInputView): Promise<void> {
    const current = currentFailedInput(value);
    if (
      !current ||
      deletionBusy.current ||
      product.access !== "authorized" ||
      product.stale ||
      reader.read !== "ready"
    ) {
      return;
    }
    deletionBusy.current = true;
    const snapshotId = region.header.source?.snapshotId;
    setInputAction({ requestId: current.requestId, state: "deleting" });
    try {
      await deleteMutation.mutateAsync({
        source: {
          agentId: agent.id,
          conversationId: current.conversationId,
          snapshotId: region.header.source?.snapshotId,
        },
        requestId: current.requestId,
      });
      reader.removeInput(current.id, current.conversationId, snapshotId);
    } catch {
      // Leave queried queue state authoritative. Repeating an explicit Delete
      // is idempotent; polling never repeats the mutation.
    } finally {
      deletionBusy.current = false;
      setInputAction(void 0);
    }
    await refreshAfterEffect();
  }
  async function answer(
    record: ConversationRecord,
    explicit?: ConversationAnswer,
  ): Promise<void> {
    const key = draftKey(record);
    const draft = currentDraft(record);
    const source = region.header.source;
    const target = region.header.native;
    if (
      !writable ||
      !source ||
      !target ||
      !record.question ||
      !record.responder ||
      record.answer ||
      !draft ||
      (draft.state !== "editing" && draft.state !== "invalid") ||
      answerBusy.current.has(key)
    ) {
      return;
    }
    const reviewed = explicit
      ? { answer: explicit, errors: {} }
      : reviewQuestionAnswer(record.question, draft);
    if (!reviewed.answer) {
      setDrafts((all) => ({
        ...all,
        [key]: { ...draft, errors: reviewed.errors },
      }));
      return;
    }
    const value = reviewed.answer;
    const requestId = crypto.randomUUID();
    answerBusy.current.add(key);
    setDrafts((all) => ({
      ...all,
      [key]: { ...draft, errors: {}, state: "sending" },
    }));
    try {
      const result = await answerMutation.mutateAsync({
        source,
        target,
        responderToken: record.responder.token,
        requestId,
        answer: value,
      });
      setDrafts((all) => ({
        ...all,
        [key]: {
          ...draft,
          state: result.delivery === "rejected" ? "rejected" : "accepted",
          answer: value,
        },
      }));
    } catch (error) {
      // A missing response does not make a single-use native responder reusable.
      const data: unknown =
        error instanceof TRPCClientError ? error.data : null;
      const invalid =
        typeof data === "object" &&
        data !== null &&
        "code" in data &&
        data.code === "BAD_REQUEST";
      setDrafts((all) => ({
        ...all,
        [key]: { ...draft, state: invalid ? "invalid" : "uncertain" },
      }));
    } finally {
      answerBusy.current.delete(key);
    }
    await refreshAfterEffect();
  }
  async function stopWork(): Promise<void> {
    const { native: source, conversationId, workId } = region.header;
    if (
      !effectAvailable ||
      !source ||
      !conversationId ||
      !workId ||
      stopBusy.current ||
      (stop?.workId === workId && stop.state !== "idle")
    ) {
      return;
    }
    stopBusy.current = true;
    const requestId = crypto.randomUUID();
    const reference = {
      workId,
      requestId,
      conversationId,
      snapshotId: source.snapshotId,
    };
    setStop({ ...reference, state: "pending" });
    try {
      const result = await stopMutation.mutateAsync({
        agentId: agent.id,
        conversationId,
        requestId,
        target: { source, workId },
      });
      setStop({ ...reference, state: result.observation });
    } catch {
      setStop({ ...reference, state: "uncertain" });
    } finally {
      stopBusy.current = false;
    }
    await refreshAfterEffect();
  }
  async function refreshAfterEffect(): Promise<void> {
    // Read failures cannot withdraw an acknowledged effect or authorize replay.
    try {
      await Promise.all([
        utils.conversations.activity.invalidate(),
        utils.conversations.messages.invalidate(),
        utils.conversations.region.invalidate(),
      ]);
    } catch {
      // The read owner publishes read failures independently of effect receipts.
    }
    reader.refresh();
    messageReader.refresh();
    recentReader.refresh();
  }
  const conversation: ConversationControls = {
    region,
    read: reader.read,
    readingOlder: reader.readingOlder,
    olderRead: reader.olderRead,
    paginationReset: reader.paginationReset,
    stop:
      stop?.workId === region.header.workId &&
      stop?.snapshotId === region.header.source?.snapshotId
        ? region.stop && region.stop.requestId === stop?.requestId
          ? region.stop.observation
          : (stop?.state ?? "idle")
        : "idle",
    drafts: Object.fromEntries(
      region.records
        .map((record) => [record.key, currentDraft(record)])
        .filter(
          (entry): entry is [string, QuestionDraft] =>
            typeof entry[0] === "string" && entry[1] !== null,
        ),
    ),
    onRead: reader.refresh,
    inputAction,
    inputDeletable:
      product.access === "authorized" &&
      !product.stale &&
      reader.read === "ready",
    onRetryInput: (value) => {
      const current = currentFailedInput(value);
      if (current && !deletionBusy.current && writable) {
        void retryInput(current);
      }
    },
    onDeleteInput: (value) => {
      void deleteInput(value);
    },
    onOlder: () => {
      floating.preserveHistory();
      reader.older();
    },
    onLatest: () => {
      reader.latest();
      floating.followLatest();
    },
    onHistory: (record) => {
      setOpened(false);
      setHistory({
        initial: recentReader.region,
        key: record?.key ?? null,
        olderOnOpen: false,
        initialFetch: recentReader.read !== "ready",
      });
    },
    onQuestionOption: (record, optionId) =>
      changeDraft(record, (draft) => ({ ...draft, optionId, errors: {} })),
    onQuestionField: (record, name, value) =>
      changeDraft(record, (draft) => {
        const fields = { ...draft.fields };
        if (value === null) {
          delete fields[name];
        } else {
          return { ...draft, fields: { ...fields, [name]: value }, errors: {} };
        }
        return { ...draft, fields, errors: {} };
      }),
    onAnswer: (record, value) => {
      void answer(record, value);
    },
  };
  return {
    product,
    agent,
    copy,
    navigation,
    accountControls,
    supplyControl: createElement(SupplyPage, {
      key: agent.id,
      agentId: agent.id,
      applied: region.header.appliedSupply,
      observation: {
        preparation: region.header.preparation,
        savedAuthenticationId: region.header.savedAuthenticationId,
        appliedAuthenticationId: region.header.appliedAuthenticationId,
      },
      unavailable: product.access !== "authorized" || product.stale,
    }),
    secondaryOverlay,
    agentNavigation: navigation?.content,
    conversation,
    messages: {
      region: messageReader.region,
      read: messageReader.read,
      olderRead: messageReader.olderRead,
      readingOlder: messageReader.readingOlder,
      paginationReset: messageReader.paginationReset,
      onRead: messageReader.refresh,
      onOlder: () => {
        floating.preserveHistory();
        messageReader.older();
      },
      onLatest: () => {
        messageReader.latest();
        floating.followLatest();
      },
    },
    recentRecords: recentReader.region,
    recentRead: recentReader.read,
    input,
    inputError: inputError ? copy.enqueueFailed : void 0,
    activity: reader.read === "error" ? "unavailable" : activity,
    writable: writable && !sendMutation.isPending,
    stoppable:
      effectAvailable && Boolean(region.header.native && region.header.workId),
    sendable: writable && !sendMutation.isPending && Boolean(input.trim()),
    status,
    statusLoading: presentation.refreshing,
    opened,
    navOpened,
    floating,
    historyOpened: history !== null,
    historyContent: history
      ? createElement(ConversationHistoryPage, {
          initial: history.initial,
          initialFocus: history.key,
          olderOnOpen: history.olderOnOpen,
          initialFetch: history.initialFetch,
          copy,
          agentName: agent.name,
        })
      : null,
    onCloseHistory: () => {
      setHistory(null);
      setOpened(true);
    },
    focus: controlled.focus,
    onOpen: () => setOpened(true),
    onClose: () => setOpened(false),
    onOpenNav: () => setNavOpened(true),
    onCloseNav: () => setNavOpened(false),
    onInput: (value) => {
      setInput(value);
      setInputError(false);
    },
    onSend: () => {
      void send();
    },
    onStop: () => {
      void stopWork();
    },
  };
}
