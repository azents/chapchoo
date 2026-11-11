"use client";
import { SheetViewport } from "@chapchoo/ui/Sheet";
import { Button } from "@mantine/core";
import { OlderRecordsContainer } from "../containers/OlderRecordsContainer";
import { ConversationBody } from "./ConversationBody";
import classes from "./ConversationHistory.module.css";
import type { HistoryScroll } from "../containers/useHistoryScroll";
import type { ChatCopy } from "../copy";
import type { ConversationControls } from "../types";
import type { ReactElement } from "react";

export interface ConversationHistoryProps {
  controls: ConversationControls;
  scroll: HistoryScroll;
  copy: ChatCopy;
  agentName: string;
}
export function ConversationHistory({
  controls,
  scroll,
  copy,
  agentName,
}: ConversationHistoryProps): ReactElement {
  return (
    <div className={classes.frame}>
      <SheetViewport
        className={classes.scroll}
        ref={scroll.viewportRef}
        role="log"
        aria-label={copy.history}
        tabIndex={0}
      >
        <div ref={scroll.contentRef} className={classes.content}>
          <OlderRecordsContainer
            controls={controls}
            viewportRef={scroll.viewportRef}
            contentRef={scroll.contentRef}
            followingLatest={!scroll.detached}
          />
          <ConversationBody
            controls={controls}
            copy={copy}
            writable={false}
            agentName={agentName}
            surface="history"
            initiallyExpandedKey={scroll.initialFocus}
          />
        </div>
      </SheetViewport>
      {scroll.unseen ? (
        <Button
          className={classes.latest}
          variant="default"
          onClick={controls.onLatest}
        >
          {copy.records.newEvents}
        </Button>
      ) : null}
    </div>
  );
}
