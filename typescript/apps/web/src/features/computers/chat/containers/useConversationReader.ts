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
} from "@chapchoo/web-core/conversation-types";

export function useConversationReader(
  initial: ConversationRegion,
  enabled: boolean,
  surface: "activity" | "history",
  stopLookup?: {
    conversationId: string;
    requestId: string;
    snapshotId: string;
  },
  initialFetch = false,
): {
  region: ConversationRegion;
  read: "ready" | "loading" | "error";
  olderRead: "ready" | "loading" | "error";
  paginationReset: object;
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
  const checkIndex = useRef(0);
  useEffect(() => {
    stopRef.current = stopLookup;
  }, [stopLookup]);
  const visibility = useDocumentVisibility();
  const [region, setRegion] = useState(initial);
  const [read, setRead] = useState<"ready" | "loading" | "error">(
    initialFetch ||
      (surface === "history" && initial.header.state === "loading")
      ? "loading"
      : "ready",
  );
  const current = useRef(initial);
  const before = useRef<ConversationAnchor | null>(null);
  const inputBefore = useRef<InputHistoryAnchor | null>(null);
  const paged = useRef(false);
  const retryHead = useRef(false);
  const [olderRead, setOlderRead] = useState<"ready" | "loading" | "error">(
    "ready",
  );
  const [readingOlder, setReadingOlder] = useState(false);
  const [paginationReset, setPaginationReset] = useState<object>({});
  const busy = useRef(false);
  const queued = useRef(false);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const refresh = useRef<() => void>(() => {});
  const active = useRef(false);
  const owner = useRef(true);
  const generation = useRef(0);
  useEffect(() => {
    owner.current = true;
    return () => {
      owner.current = false;
    };
  }, []);
  useEffect(() => {
    const lifetime = ++generation.current;
    active.current = enabled && visibility === "visible";
    function hasOwner(): boolean {
      return owner.current && lifetime === generation.current;
    }
    function canRead(): boolean {
      return hasOwner() && active.current;
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
      // Failed page reads yield the next cadence to the current head. Retain
      // the page request for the following cadence instead of starving live data.
      const anchor = retryHead.current ? null : before.current;
      const inputAnchor = retryHead.current ? null : inputBefore.current;
      const snapshot = current.current.header.source?.snapshotId;
      try {
        const known = (current.current.inputs ?? [])
          .filter((item) => !item.deleted)
          .map((item) => item.id);
        const offset = checkIndex.current % Math.max(known.length, 1);
        const rotating = [...known.slice(offset), ...known.slice(0, offset)];
        checkIndex.current = offset + 25;
        const checkInputs = rotating.slice(0, 25);
        const input = {
          agentId: initial.header.agentId,
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
        };
        const incoming =
          surface === "activity"
            ? await utils.conversations.activity.fetch(input, {
                staleTime: 0,
                retry: false,
              })
            : await utils.conversations.region.fetch(
                {
                  ...input,
                  before: anchor
                    ? {
                        ordinal: String(anchor.ordinal),
                        index: String(anchor.index),
                      }
                    : void 0,
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
        if (changedSource) {
          paged.current = false;
          retryHead.current = false;
          setOlderRead("ready");
          before.current = null;
          inputBefore.current = null;
          setReadingOlder(false);
          current.current = incoming;
          setRegion(current.current);
        } else if (
          (!anchor && !inputAnchor) ||
          (anchor === before.current && inputAnchor === inputBefore.current)
        ) {
          const previous = current.current;
          const merged = mergeConversationRegion(previous, incoming);
          // A missing page cursor requests the latest backend window. Once a
          // stream is exhausted, that window must not revive its older cursor.
          current.current = {
            ...merged,
            records: surface === "activity" ? incoming.records : merged.records,
            older: anchor
              ? incoming.older
              : paged.current || inputBefore.current || before.current
                ? previous.older
                : incoming.older,
            inputOlder: inputAnchor
              ? incoming.inputOlder
              : paged.current || inputBefore.current || before.current
                ? previous.inputOlder
                : incoming.inputOlder,
          };
          if (anchor || inputAnchor) {
            setOlderRead("ready");
            paged.current = true;
            before.current = null;
            inputBefore.current = null;
            setReadingOlder(false);
          }
          setRegion(current.current);
        }
        retryHead.current = false;
        if (
          (!anchor && !inputAnchor) ||
          changedSource ||
          incoming.header.state === "loading"
        ) {
          setRead(
            incoming.refill ||
              (surface === "history" && incoming.header.state === "loading")
              ? "loading"
              : "ready",
          );
        }
        if (incoming.refill) {
          queued.current = true;
        }
      } catch {
        if (
          (anchor || inputAnchor) &&
          (anchor !== before.current || inputAnchor !== inputBefore.current)
        ) {
          return;
        }
        retryHead.current = Boolean(anchor || inputAnchor);
        if (hasOwner()) {
          if (anchor || inputAnchor) {
            setOlderRead("error");
          } else {
            setRead("error");
          }
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
        } else if (owner.current && active.current) {
          // Continue with the current owner after a withdrawn read settles.
          refresh.current();
        }
      }
    }
    refresh.current = () => {
      void run();
    };
    // Hydrate the exact SSR region; first active continuation starts on cadence, not mount.
    if (active.current) {
      timer.current = setTimeout(
        () => refresh.current(),
        initialFetch ? 0 : 1000,
      );
    }
    return () => {
      generation.current += 1;
      active.current = false;
      clearTimeout(timer.current ?? void 0);
    };
  }, [
    enabled,
    visibility,
    initial.header.agentId,
    utils,
    surface,
    initialFetch,
  ]);
  useWindowEvent("online", () => refresh.current());
  useWindowEvent("focus", () => refresh.current());
  const request = useCallback((): void => refresh.current(), []);
  return {
    region,
    read,
    olderRead,
    paginationReset,
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
      const recordAnchor =
        surface === "history" ? (current.current.older ?? null) : null;
      if (!recordAnchor && !current.current.inputOlder) {
        return;
      }
      // Admit the page even during an in-flight latest poll. run() queues it;
      // the stale latest result cannot consume or replace the page request.
      before.current = recordAnchor;
      inputBefore.current = current.current.inputOlder ?? null;
      setReadingOlder(true);
      setOlderRead("loading");
      refresh.current();
    },
    latest: () => {
      before.current = null;
      inputBefore.current = null;
      retryHead.current = false;
      setOlderRead("ready");
      setPaginationReset({});
      setReadingOlder(false);
      queued.current = true;
      refresh.current();
    },
  };
}
