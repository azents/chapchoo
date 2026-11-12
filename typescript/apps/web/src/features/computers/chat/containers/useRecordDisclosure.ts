"use client";
import { useRef, useState } from "react";
import type { ChatCopy } from "../copy";
import type { ConversationRecord } from "@chapchoo/web-core/conversation-types";
import type { MouseEvent, ReactNode, RefObject } from "react";
export interface RecordDisclosureInput {
  record: ConversationRecord;
  copy: ChatCopy;
  children: ReactNode;
  initiallyOpened?: boolean;
}
export interface RecordDisclosure {
  opened: boolean;
  setOpened: (value: boolean) => void;
  headerRef: RefObject<HTMLButtonElement | null>;
  close: () => void;
  closeFromBody: (event: MouseEvent<HTMLDivElement>) => void;
}
export function useRecordDisclosure(
  props: RecordDisclosureInput,
): RecordDisclosureInput & { disclosure: RecordDisclosure } {
  const [opened, setOpened] = useState(props.initiallyOpened ?? false);
  const tool = props.record.kind === "tool" || props.record.kind === "terminal";
  const headerRef = useRef<HTMLButtonElement>(null);
  function close(): void {
    setOpened(false);
    headerRef.current?.focus({ preventScroll: true });
  }
  return {
    ...props,
    disclosure: {
      opened,
      setOpened,
      headerRef,
      close,
      closeFromBody: (event): void => {
        if (!tool || window.getSelection()?.toString()) {
          return;
        }
        if (
          event.target instanceof Element &&
          event.target.closest(
            'a,button,input,textarea,select,summary,audio,video,[role="button"],[contenteditable="true"]',
          )
        ) {
          return;
        }
        close();
      },
    },
  };
}
