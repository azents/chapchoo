"use client";
import { reconcileLocalConversationInputs } from "@chapchoo/web-core/conversation-state";
import { useEffect, useRef } from "react";
import { useConversationReader } from "./useConversationReader";
import { useHistoryScroll } from "./useHistoryScroll";
import type { ConversationHistoryProps } from "../components/ConversationHistory";
import type { ChatCopy } from "../copy";
import type { ConversationControls } from "../types";
import type { ConversationRegion } from "@chapchoo/web-core/conversation-types";

export interface ConversationHistoryInput {
  initial: ConversationRegion;
  initialFocus: string | null;
  olderOnOpen: boolean;
  agentName: string;
  copy: ChatCopy;
  inputs: ConversationControls["inputs"];
  drafts: ConversationControls["drafts"];
  deletions: { id: string; conversationId: string; snapshotId?: string }[];
}
export function useConversationHistory(
  props: ConversationHistoryInput,
): ConversationHistoryProps {
  const reader = useConversationReader(
    props.initial,
    true,
    void 0,
    props.inputs,
  );
  const scroll = useHistoryScroll(props.initialFocus, reader.region);
  const { markInputDeleted } = reader;
  const appliedDeletions = useRef(new Set<string>());
  useEffect(() => {
    for (const deletion of props.deletions) {
      const key = JSON.stringify(deletion);
      if (!appliedDeletions.current.has(key)) {
        appliedDeletions.current.add(key);
        markInputDeleted(
          deletion.id,
          deletion.conversationId,
          deletion.snapshotId,
        );
      }
    }
  }, [props.deletions, markInputDeleted]);
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
      inputs: reconcileLocalConversationInputs(
        props.inputs.filter(
          (input) =>
            (input.conversationId === void 0
              ? reader.region.header.conversationId ===
                props.initial.header.conversationId
              : input.conversationId === reader.region.header.conversationId) &&
            input.snapshotId === reader.region.header.source?.snapshotId,
        ),
        reader.region.inputs,
        reader.region.closedInputs,
      ),
      drafts: props.drafts,
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
