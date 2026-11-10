"use client";
import { HeaderControlsContext } from "@chapchoo/ui/AppHeader";
import { AppNavigationContext } from "@chapchoo/ui/AppNavigation";
import { computerOperation } from "@chapchoo/web-core/computer-messages";
import { conversationActivity } from "@chapchoo/web-core/conversation-activity";
import {
  initialQuestionDraft,
  reviewQuestionAnswer,
} from "@chapchoo/web-core/conversation-form";
import { reconcileLocalConversationInputs } from "@chapchoo/web-core/conversation-state";
import { useDisplay } from "@chapchoo/web-core/display-context";
import { TRPCClientError } from "@trpc/client";
import { useLocale } from "next-intl";
import { createElement, useContext, useEffect, useRef, useState } from "react";
import { trpc } from "@/shared/rpc/client";
import { useAgentsContainer } from "../../containers/useAgentsContainer";
import { ConversationHistoryPage } from "../ConversationHistoryPage";
import { activityStatus, chatCopy } from "../copy";
import { SupplyPage } from "../SupplyPage";
import { useChatFocus } from "./useChatFocus";
import { useConversationReader } from "./useConversationReader";
import { useFloatingChat } from "./useFloatingChat";
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
  } | null>(null);
  const [navOpened, setNavOpened] = useState(false);
  const [inputs, setInputs] = useState<ConversationControls["inputs"]>([]);
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
    stop ?? void 0,
  );
  const region = reader.region;
  const previousSnapshot = useRef(
    props.initialConversation.header.source?.snapshotId ?? null,
  );
  useEffect(() => {
    const snapshot = region.header.source?.snapshotId ?? null;
    const previous = previousSnapshot.current;
    previousSnapshot.current = snapshot;
    if (previous !== snapshot) {
      setInputs((all) =>
        all
          .filter((item) => previous === null || item.snapshotId === snapshot)
          .map((item) => ({
            ...item,
            snapshotId: item.snapshotId ?? snapshot ?? void 0,
          })),
      );
    }
    const recovered = inputs.find(
      (item) =>
        item.observation === "uncertain" &&
        region.inputs?.some(
          (retained) => retained.requestId === item.requestId,
        ),
    );
    if (recovered) {
      setInput((value) => (value === recovered.text ? "" : value));
      setInputs((all) => reconcileLocalConversationInputs(all, region.inputs));
    }
  }, [region.header.source?.snapshotId, region.inputs, inputs]);
  const floating = useFloatingChat(
    input,
    `${props.initial.type === "detail" ? props.initial.agent.id : ""}:${region.header.source?.snapshotId ?? ""}`,
    product.access === "authorized",
    region.records.filter(
      (record) =>
        record.kind === "user" ||
        record.kind === "assistant" ||
        record.kind === "request",
    ),
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
  const activity = conversationActivity(region, inputs);
  const writable =
    effectAvailable &&
    reader.read === "ready" &&
    region.header.state !== "loading" &&
    activity !== "authenticationRequired";
  const operation = computerOperation(computer, product.copy);
  const status =
    product.error ??
    (product.stale
      ? product.copy.stale
      : computer.phase !== "completed"
        ? operation.label
        : computer.power === "stopped"
          ? `${product.detailCopy.computer} · ${product.copy.stopped}`
          : computer.power === "unknown"
            ? `${product.detailCopy.computer} · ${product.copy.unknown}`
            : computer.connector !== "connected"
              ? product.copy[computer.connector]
              : reader.read === "error"
                ? copy.error
                : reader.read === "loading"
                  ? copy.loading
                  : activityStatus(copy, activity));
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
  async function send(retryText?: string): Promise<void> {
    const text = retryText ?? input;
    if (
      !writable ||
      inputBusy.current ||
      !text.trim() ||
      inputs.some(
        (item) => item.text === text && item.observation === "uncertain",
      )
    ) {
      return;
    }
    inputBusy.current = true;
    const requestId = crypto.randomUUID();
    const target = {
      agentId: agent.id,
      conversationId: region.header.conversationId,
      snapshotId: region.header.source?.snapshotId,
      requestId,
      text,
    };
    setInputs((all) => [
      ...all,
      {
        text,
        requestId,
        observation: "sending",
        snapshotId: target.snapshotId,
      },
    ]);
    try {
      const receipt = await sendMutation.mutateAsync(target);
      setInputs((all) =>
        all.map((item) =>
          item.requestId === requestId
            ? {
                ...item,
                observation: "accepted",
                conversationId: receipt.conversationId,
              }
            : item,
        ),
      );
      if (retryText === void 0) {
        setInput((current) => (current === text ? "" : current));
      }
      floating.followLatest();
    } catch {
      setInputs((all) =>
        all.map((item) =>
          item.requestId === requestId
            ? { ...item, observation: "uncertain" }
            : item,
        ),
      );
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
          item.observation === "failed" &&
          !item.deleted,
      ) ?? null
    );
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
      reader.markInputDeleted(current.id, current.conversationId, snapshotId);
      setInputAction(void 0);
    } catch {
      // Lost delete responses recover through read-only tombstone observations.
      setInputAction({ requestId: current.requestId, state: "uncertain" });
    } finally {
      deletionBusy.current = false;
    }
    await refreshAfterEffect();
  }
  async function recoverDeletion(): Promise<void> {
    if (inputAction?.state === "uncertain") {
      const current = region.inputs?.find(
        (item) => item.requestId === inputAction.requestId,
      );
      if (current && current.conversationId === region.header.conversationId) {
        const snapshot = region.header.source?.snapshotId;
        try {
          const result = await utils.conversations.inputResult.fetch(
            {
              agentId: agent.id,
              conversationId: current.conversationId,
              requestId: current.requestId,
            },
            { staleTime: 0, retry: false },
          );
          if (result.deleted) {
            reader.markInputDeleted(
              current.id,
              current.conversationId,
              snapshot,
            );
          }
          setInputAction(void 0);
        } catch {
          // Keep uncertainty until the exact operation can be read.
        }
      }
    }
    reader.refresh();
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
      await utils.conversations.region.invalidate();
    } catch {
      // The read owner publishes read failures independently of effect receipts.
    }
    reader.refresh();
  }
  const conversation: ConversationControls = {
    region,
    read: reader.read,
    readingOlder: reader.readingOlder,
    inputs: inputs.filter(
      (item) =>
        !region.inputs?.some(
          (retained) => retained.requestId === item.requestId,
        ) &&
        (item.snapshotId === void 0 ||
          item.snapshotId === region.header.source?.snapshotId),
    ),
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
    onRead: () => {
      void recoverDeletion();
    },
    inputAction,
    inputDeletable:
      product.access === "authorized" &&
      !product.stale &&
      reader.read === "ready",
    onRetryInput: (value) => {
      const current = currentFailedInput(value);
      if (current?.text && inputAction?.state !== "deleting") {
        void send(current.text);
      }
    },
    onDeleteInput: (value) => {
      void deleteInput(value);
    },
    onOlder: () => {
      setHistory({
        initial: region,
        key: region.records[0]?.key ?? null,
        olderOnOpen: true,
      });
    },
    onLatest: () => {
      reader.latest();
      floating.followLatest();
    },
    onHistory: (record) => {
      setOpened(false);
      setHistory({
        initial: region,
        key: record?.key ?? null,
        olderOnOpen: false,
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
    input,
    activity: reader.read === "error" ? "unavailable" : activity,
    writable: writable && !sendMutation.isPending,
    stoppable:
      effectAvailable && Boolean(region.header.native && region.header.workId),
    sendable:
      writable &&
      !sendMutation.isPending &&
      Boolean(input.trim()) &&
      !inputs.some(
        (item) => item.text === input && item.observation === "uncertain",
      ),
    status,
    opened,
    navOpened,
    floating,
    historyOpened: history !== null,
    historyContent: history
      ? createElement(ConversationHistoryPage, {
          initial: history.initial,
          initialFocus: history.key,
          olderOnOpen: history.olderOnOpen,
          deletions: (region.inputs ?? [])
            .filter((item) => item.deleted)
            .map((item) => ({
              id: item.id,
              conversationId: item.conversationId,
              snapshotId: region.header.source?.snapshotId,
            })),
          copy,
          agentName: agent.name,
          inputs,
          drafts: conversation.drafts,
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
    onInput: setInput,
    onSend: () => {
      void send();
    },
    onStop: () => {
      void stopWork();
    },
  };
}
