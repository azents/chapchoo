"use client";
import { useOlderRecords } from "./useOlderRecords";
import type { ConversationControls } from "../types";
import type { RefObject } from "react";
export function OlderRecordsContainer({
  controls,
  viewportRef,
  contentRef,
}: {
  controls: ConversationControls;
  viewportRef: RefObject<HTMLDivElement | null>;
  contentRef: RefObject<HTMLDivElement | null>;
}): null {
  useOlderRecords(controls, viewportRef, contentRef);
  return null;
}
