"use client";
import { mergeConversationMessages } from "@chapchoo/web-core/conversation-message-state";
import { useDocumentVisibility, useWindowEvent } from "@mantine/hooks";
import { useCallback, useEffect, useRef, useState } from "react";
import { trpc } from "@/shared/rpc/client";
import type { ConversationMessageRegion } from "@chapchoo/web-core/conversation-message-types";

export interface MessageReader {
  region: ConversationMessageRegion;
  read: "ready" | "loading" | "error";
  olderRead: "ready" | "loading" | "error";
  readingOlder: boolean;
  paginationReset: object;
  refresh: () => void;
  older: () => void;
  latest: () => void;
}

/** The conversation receipt frontier is unrelated to native source ordinals. */
export function useMessageReader(
  initial: ConversationMessageRegion,
  enabled: boolean,
): MessageReader {
  const utils = trpc.useUtils();
  const visibility = useDocumentVisibility();
  const [region, setRegion] = useState(initial);
  const current = useRef(initial);
  const [read, setRead] = useState<MessageReader["read"]>("ready");
  const [olderRead, setOlderRead] =
    useState<MessageReader["olderRead"]>("ready");
  const [readingOlder, setReadingOlder] = useState(false);
  const [paginationReset, setPaginationReset] = useState<object>({});
  const before = useRef<bigint | null>(null);
  const latest = useRef(false);
  const retryHead = useRef(false);
  const busy = useRef(false);
  const queued = useRef(false);
  const generation = useRef(0);
  const runNow = useRef<() => void>(() => {});

  useEffect(() => {
    const lifetime = ++generation.current;
    const active = enabled && visibility === "visible";
    let timer: ReturnType<typeof setTimeout> | null = null;
    if (current.current.agentId !== initial.agentId) {
      current.current = initial;
      setRegion(initial);
      before.current = null;
      latest.current = false;
      retryHead.current = false;
      setRead("ready");
      setOlderRead("ready");
      setReadingOlder(false);
      setPaginationReset({});
    }
    function canRead(): boolean {
      return active && generation.current === lifetime;
    }
    async function run(): Promise<void> {
      if (!canRead()) {
        return;
      }
      if (busy.current) {
        queued.current = true;
        return;
      }
      clearTimeout(timer ?? void 0);
      busy.current = true;
      const requestedLatest = latest.current;
      const anchor =
        requestedLatest || retryHead.current ? null : before.current;
      const previous = current.current;
      try {
        const window = requestedLatest
          ? { type: "latest" as const }
          : anchor !== null
            ? { type: "before" as const, order: String(anchor) }
            : {
                type: "after" as const,
                order: String(previous.next?.order ?? previous.through),
                through: previous.next ? String(previous.next.through) : void 0,
              };
        const incoming = await utils.conversations.messages.fetch(
          {
            agentId: initial.agentId,
            expectedConversationId: previous.conversationId ?? null,
            window,
          },
          { staleTime: 0, retry: false },
        );
        if (!canRead()) {
          return;
        }
        const changed =
          incoming.conversationId !== current.current.conversationId;
        if (changed) {
          before.current = null;
          latest.current = false;
          retryHead.current = false;
          setPaginationReset({});
          setOlderRead("ready");
          setReadingOlder(false);
          current.current = incoming;
        } else if (anchor !== null && latest.current) {
          // A user-selected latest reset supersedes an in-flight older read.
          queued.current = true;
          return;
        } else {
          current.current = mergeConversationMessages(
            current.current,
            incoming,
          );
          if (anchor !== null && before.current === anchor) {
            before.current = null;
            setOlderRead("ready");
            setReadingOlder(false);
          }
          if (requestedLatest) {
            latest.current = false;
          }
        }
        setRegion(current.current);
        if (anchor === null) {
          setRead("ready");
          retryHead.current = false;
        }
        if (current.current.next || before.current !== null || latest.current) {
          queued.current = true;
        }
      } catch {
        if (!canRead()) {
          return;
        }
        if (anchor !== null) {
          setOlderRead("error");
          setReadingOlder(false);
          retryHead.current = true;
        } else {
          setRead("error");
        }
      } finally {
        busy.current = false;
        if (canRead()) {
          const immediate = queued.current;
          queued.current = false;
          timer = setTimeout(
            () => {
              void run();
            },
            immediate ? 0 : 1000,
          );
        } else if (generation.current !== lifetime) {
          // The replacement owner may have queued behind this stale request.
          // Its own visibility/authorization check still governs continuation.
          runNow.current();
        }
      }
    }
    runNow.current = () => {
      void run();
    };
    // Hydration starts with the rendered page, not a duplicate immediate read.
    if (active) {
      timer = setTimeout(() => {
        void run();
      }, 1000);
    }
    return () => {
      generation.current += 1;
      clearTimeout(timer ?? void 0);
      runNow.current = () => {};
    };
  }, [enabled, visibility, initial, utils]);

  const refresh = useCallback((): void => {
    runNow.current();
  }, []);
  useWindowEvent("focus", refresh);
  useWindowEvent("online", refresh);
  return {
    region: region.agentId === initial.agentId ? region : initial,
    read,
    olderRead,
    readingOlder,
    paginationReset,
    refresh,
    older: useCallback((): void => {
      if (current.current.older === void 0 || before.current !== null) {
        return;
      }
      before.current = current.current.older;
      setReadingOlder(true);
      setOlderRead("loading");
      runNow.current();
    }, []),
    latest: useCallback((): void => {
      latest.current = true;
      before.current = null;
      retryHead.current = false;
      setReadingOlder(false);
      setOlderRead("ready");
      setPaginationReset({});
      runNow.current();
    }, []),
  };
}
