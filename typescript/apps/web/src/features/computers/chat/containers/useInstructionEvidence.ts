"use client";
import { formatDisplayDate } from "@chapchoo/ui/displayFormatting";
import { useAuth } from "@chapchoo/web-core/auth-context";
import { useDisplay } from "@chapchoo/web-core/display-context";
import { useLocale } from "next-intl";
import { useEffect, useRef, useState } from "react";
import { trpc } from "@/shared/rpc/client";
import { instructionEvidenceCopy } from "../instructionEvidenceCopy";
import type {
  InstructionEvidenceInput,
  InstructionEvidenceProps,
  InstructionEvidenceState,
} from "../instructionEvidenceTypes";
import type {
  InstructionHistoryItem,
  InstructionHistoryPage,
} from "@chapchoo/web-core/instruction-evidence";

type Continuation = {
  type: "ready" | "loading" | "error";
  pages: InstructionHistoryPage[];
};
export function useInstructionEvidence(
  input: InstructionEvidenceInput,
): InstructionEvidenceProps {
  const auth = useAuth();
  const authorized = auth.state.type === "AUTHENTICATED";
  const [opened, setOpened] = useState(false);
  const [selection, setSelection] = useState<string | null>(null);
  const [continuation, setContinuation] = useState<Continuation>({
    type: "ready",
    pages: [],
  });
  const loadingOlder = useRef(false);
  const fence = useRef(0);
  useEffect(
    () => () => {
      fence.current += 1;
    },
    [input.agentId, input.conversationId, authorized],
  );
  const copy = instructionEvidenceCopy(useLocale());
  const { value: display } = useDisplay();
  const utils = trpc.useUtils();
  const history = trpc.conversations.instructionHistory.useQuery(input, {
    enabled: opened && authorized,
    retry: false,
    refetchOnWindowFocus: false,
  });
  const items = history.data
    ? Array.from(
        [history.data, ...continuation.pages]
          .reduce((all, page) => {
            for (const item of page.items) {
              if (!all.has(item.operationId)) {
                all.set(item.operationId, item);
              }
            }
            return all;
          }, new Map<string, InstructionHistoryItem>())
          .values(),
      )
    : null;
  const nextBefore = continuation.pages.length
    ? continuation.pages.at(-1)?.nextBefore
    : history.data?.nextBefore;
  const selected =
    items?.find((item) => item.operationId === selection)?.operationId ??
    items?.[0]?.operationId ??
    "";
  const evidence = trpc.conversations.instructionEvidence.useQuery(
    { ...input, operationId: selected },
    { enabled: opened && authorized && selected !== "", retry: false },
  );
  async function more(): Promise<void> {
    if (loadingOlder.current || !nextBefore || !authorized) {
      return;
    }
    const cursor = nextBefore;
    const operation = fence.current;
    loadingOlder.current = true;
    setContinuation((current) => ({ ...current, type: "loading" }));
    try {
      const page = await utils.conversations.instructionHistory.fetch({
        ...input,
        beforeOperationId: cursor,
      });
      if (page.nextBefore === cursor) {
        throw new Error("Instruction history cursor did not advance");
      }
      if (operation === fence.current) {
        setContinuation((current) => ({
          type: "ready",
          pages: [...current.pages, page],
        }));
      }
    } catch {
      if (operation === fence.current) {
        setContinuation((current) => ({ ...current, type: "error" }));
      }
    } finally {
      loadingOlder.current = false;
    }
  }
  const state: InstructionEvidenceState =
    !authorized || history.isError
      ? { type: "error" }
      : !items
        ? { type: "loading" }
        : items.length === 0
          ? { type: "empty" }
          : {
              type: "ready",
              selected,
              options: items.map((item) => ({
                value: item.operationId,
                label: `${formatDisplayDate(item.createdAt, display.region, display.timeZone, "medium") ?? item.createdAt} · ${item.appliedAt === void 0 ? copy.prepared : copy.supplied} · ${item.executionId} · ${item.operationId}`,
              })),
              more: { available: Boolean(nextBefore), read: continuation.type },
              evidence: evidence.isError
                ? { type: "error" }
                : evidence.data
                  ? { type: "ready", value: evidence.data }
                  : { type: "loading" },
            };
  return {
    copy,
    opened,
    state,
    onToggle: () => setOpened((value) => !value),
    onSelect: setSelection,
    onMore: () => {
      void more();
    },
    onRead: () => {
      void utils.conversations.instructionHistory.invalidate(input);
      if (selected) {
        void utils.conversations.instructionEvidence.invalidate({
          ...input,
          operationId: selected,
        });
      }
    },
  };
}
