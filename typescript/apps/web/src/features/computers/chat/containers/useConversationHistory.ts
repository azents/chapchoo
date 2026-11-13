"use client";
import { useEffect, useRef } from "react";
import { useConversationReader } from "./useConversationReader";
import { useHistoryScroll } from "./useHistoryScroll";
import type { ConversationHistoryProps } from "../components/ConversationHistory";
import type { ChatCopy } from "../copy";
import type { ConversationRegion } from "@chapchoo/web-core/conversation-types";

export interface ConversationHistoryInput {
  initial: ConversationRegion;
  initialFocus: string | null;
  olderOnOpen: boolean;
  initialFetch?: boolean;
  agentName: string;
  copy: ChatCopy;
}
export function useConversationHistory(
  props: ConversationHistoryInput,
): ConversationHistoryProps {
  const reader = useConversationReader(
    props.initial,
    true,
    "history",
    void 0,
    props.initialFetch,
  );
  const scroll = useHistoryScroll(props.initialFocus, reader.region);
  const older = useRef((): void => {});
  older.current = () => {
    scroll.preserveHistory();
    reader.older();
  };
  useEffect(() => {
    if (props.olderOnOpen) {
      older.current();
    }
  }, [props.olderOnOpen]);
  return {
    copy: props.copy,
    agentName: props.agentName,
    scroll,
    controls: {
      region: reader.region,
      read: reader.read,
      readingOlder: reader.readingOlder,
      olderRead: reader.olderRead,
      paginationReset: reader.paginationReset,
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
    },
  };
}
