"use client";
import {
  markConversationInputDeleted,
  mergeConversationRegion,
} from "@chapchoo/web-core/conversation-state";
import { useDocumentVisibility, useWindowEvent } from "@mantine/hooks";
import { useCallback, useEffect, useRef, useState } from "react";
import { trpc } from "@/shared/rpc/client";
import type {
  ConversationAnchor,
  ConversationRegion,
  InputHistoryAnchor,
  LocalConversationInput,
} from "@chapchoo/web-core/conversation-types";

export function useConversationReader(
  initial: ConversationRegion,
  enabled: boolean,
  stopLookup?: {
    conversationId: string;
    requestId: string;
    snapshotId: string;
  },
  pendingInputs: LocalConversationInput[] = [],
): {
  region: ConversationRegion;
  read: "ready" | "loading" | "error";
  readingOlder: boolean;
  refresh: () => void;
  older: () => void;
  latest: () => void;
  markInputDeleted: (
    id: string,
    conversationId: string,
    snapshotId?: string,
  ) => void;
} {
  const utils = trpc.useUtils();
  const stopRef = useRef(stopLookup);
  const pendingRef = useRef(pendingInputs);
  const checkIndex = useRef(0);
  useEffect(() => {
    pendingRef.current = pendingInputs;
  }, [pendingInputs]);
  useEffect(() => {
    stopRef.current = stopLookup;
  }, [stopLookup]);
  const visibility = useDocumentVisibility();
  const [region, setRegion] = useState(initial);
  const [read, setRead] = useState<"ready" | "loading" | "error">(
    initial.header.state === "loading" ? "loading" : "ready",
  );
  const current = useRef(initial);
  const before = useRef<ConversationAnchor | null>(null);
  const inputBefore = useRef<InputHistoryAnchor | null>(null);
  const [readingOlder, setReadingOlder] = useState(false);
  const busy = useRef(false);
  const queued = useRef(false);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const refresh = useRef<() => void>(() => {});
  const active = useRef(false);
  const owner = useRef(true);
  useEffect(() => {
    owner.current = true;
    return () => {
      owner.current = false;
    };
  }, []);
  useEffect(() => {
    active.current = enabled && visibility === "visible";
    function hasOwner(): boolean {
      return owner.current;
    }
    function canRead(): boolean {
      return owner.current && active.current;
    }
    async function run(): Promise<void> {
      if (!canRead()) {
        return;
      }
      if (busy.current) {
        queued.current = true;
        return;
      }
      clearTimeout(timer.current ?? void 0);
      busy.current = true;
      const anchor = before.current;
      const inputAnchor = inputBefore.current;
      const snapshot = current.current.header.source?.snapshotId;
      try {
        const known = (current.current.inputs ?? [])
          .filter((item) => !item.deleted)
          .map((item) => item.id);
        const offset = checkIndex.current % Math.max(known.length, 1);
        const rotating = [...known.slice(offset), ...known.slice(0, offset)];
        checkIndex.current = offset + 25;
        const checkInputs = [
          ...new Set([
            ...pendingRef.current
              .filter(
                (item) =>
                  item.conversationId === current.current.header.conversationId,
              )
              .flatMap((item) => (item.id ? [item.id] : [])),
            ...rotating,
          ]),
        ].slice(0, 25);
        const incoming = await utils.conversations.region.fetch(
          {
            agentId: initial.header.agentId,
            before: anchor
              ? { ordinal: String(anchor.ordinal), index: String(anchor.index) }
              : void 0,
            inputBefore: inputAnchor
              ? {
                  acceptedAt: String(inputAnchor.acceptedAt),
                  requestId: inputAnchor.requestId,
                }
              : void 0,
            knownInputs: (current.current.inputs ?? [])
              .filter(
                (input) =>
                  !inputAnchor ||
                  input.acceptedAt < inputAnchor.acceptedAt ||
                  (input.acceptedAt === inputAnchor.acceptedAt &&
                    input.requestId < inputAnchor.requestId),
              )
              .slice(-24)
              .filter((input) => input.text !== void 0)
              .map((input) => input.id),
            expectedSnapshotId: snapshot,
            expectedConversationId: current.current.header.conversationId,
            checkInputs,
            stop: stopRef.current,
            known: current.current.records
              .filter(
                (record) =>
                  !anchor ||
                  record.anchor.ordinal < anchor.ordinal ||
                  (record.anchor.ordinal === anchor.ordinal &&
                    record.anchor.index < anchor.index),
              )
              .slice(-24)
              .map((record) => ({
                key: record.key,
                updatedOrdinal: String(record.updatedOrdinal),
              })),
          },
          { staleTime: 0, retry: false },
        );
        if (!hasOwner()) {
          return;
        }
        const changedSource =
          incoming.header.conversationId !==
            current.current.header.conversationId ||
          incoming.header.source?.snapshotId !==
            current.current.header.source?.snapshotId;
        if (changedSource || incoming.header.state === "loading") {
          before.current = null;
          inputBefore.current = null;
          setReadingOlder(false);
          current.current = incoming;
          setRegion(incoming);
        } else if (
          anchor === before.current &&
          inputAnchor === inputBefore.current
        ) {
          current.current = mergeConversationRegion(current.current, incoming);
          setRegion(current.current);
        }
        setRead(
          incoming.refill || incoming.header.state === "loading"
            ? "loading"
            : "ready",
        );
        if (incoming.refill) {
          queued.current = true;
        }
      } catch {
        if (hasOwner()) {
          setRead("error");
        }
      } finally {
        busy.current = false;
        if (canRead()) {
          const immediate = queued.current;
          queued.current = false;
          timer.current = setTimeout(
            () => refresh.current(),
            immediate ? 0 : 1000,
          );
        }
      }
    }
    refresh.current = () => {
      void run();
    };
    // Hydrate the exact SSR region; first active continuation starts on cadence, not mount.
    if (active.current) {
      timer.current = setTimeout(() => refresh.current(), 1000);
    }
    return () => {
      active.current = false;
      clearTimeout(timer.current ?? void 0);
    };
  }, [enabled, visibility, initial.header.agentId, utils]);
  useWindowEvent("online", () => refresh.current());
  useWindowEvent("focus", () => refresh.current());
  const request = useCallback((): void => refresh.current(), []);
  return {
    region,
    read,
    readingOlder,
    refresh: request,
    markInputDeleted: (id, conversationId, snapshotId) => {
      current.current = markConversationInputDeleted(
        current.current,
        id,
        conversationId,
        snapshotId,
      );
      setRegion(current.current);
    },
    older: () => {
      if (
        (!current.current.older && !current.current.inputOlder) ||
        busy.current
      ) {
        return;
      }
      before.current = current.current.older ?? before.current;
      inputBefore.current = current.current.inputOlder ?? inputBefore.current;
      setReadingOlder(true);
      setRead("loading");
      refresh.current();
    },
    latest: () => {
      before.current = null;
      inputBefore.current = null;
      setReadingOlder(false);
      queued.current = true;
      refresh.current();
    },
  };
}
